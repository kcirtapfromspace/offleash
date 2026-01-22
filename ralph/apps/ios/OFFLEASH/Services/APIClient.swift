import Foundation
import Security
import CommonCrypto
import FirebaseCrashlytics

// MARK: - Auth State Notifications

extension Notification.Name {
    static let authStateChanged = Notification.Name("com.offleash.authStateChanged")
}

// MARK: - HTTP Methods

enum HTTPMethod: String {
    case get = "GET"
    case post = "POST"
    case put = "PUT"
    case delete = "DELETE"
}

// MARK: - API Errors

enum APIError: Error, LocalizedError {
    case invalidURL
    case invalidResponse
    case httpError(statusCode: Int, message: String?)
    case decodingError(Error)
    case encodingError(Error)
    case networkError(Error)
    case unauthorized
    case noData
    case certificatePinningFailed

    var errorDescription: String? {
        switch self {
        case .invalidURL:
            return "Invalid URL"
        case .invalidResponse:
            return "Invalid response from server"
        case .httpError(let statusCode, let message):
            return "HTTP error \(statusCode): \(message ?? "Unknown error")"
        case .decodingError(let error):
            return "Failed to decode response: \(error.localizedDescription)"
        case .encodingError(let error):
            return "Failed to encode request: \(error.localizedDescription)"
        case .networkError(let error):
            return "Network error: \(error.localizedDescription)"
        case .unauthorized:
            return "Unauthorized - please log in again"
        case .noData:
            return "No data received from server"
        case .certificatePinningFailed:
            return "Certificate pinning validation failed - possible security threat"
        }
    }
}

// MARK: - API Response Wrapper

struct APIResponse<T: Decodable>: Decodable {
    let data: T?
    let error: String?
    let message: String?
}

// MARK: - Keychain Helper

final class KeychainHelper: @unchecked Sendable {
    static let shared = KeychainHelper()

    private let service = "com.offleash.app"
    private let tokenKey = "authToken"

    private init() {}

    func saveToken(_ token: String) -> Bool {
        guard let data = token.data(using: .utf8) else { return false }

        // Delete existing token first
        deleteToken()

        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: tokenKey,
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        ]

        let status = SecItemAdd(query as CFDictionary, nil)
        return status == errSecSuccess
    }

    func getToken() -> String? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: tokenKey,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne
        ]

        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        guard status == errSecSuccess,
              let data = result as? Data,
              let token = String(data: data, encoding: .utf8) else {
            return nil
        }

        return token
    }

    @discardableResult
    func deleteToken() -> Bool {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: tokenKey
        ]

        let status = SecItemDelete(query as CFDictionary)
        return status == errSecSuccess || status == errSecItemNotFound
    }

    var hasToken: Bool {
        getToken() != nil
    }
}

// MARK: - Certificate Pinning

/// URLSessionDelegate that implements SSL certificate pinning for MITM attack prevention.
///
/// ## Certificate Rotation Procedure
///
/// When rotating certificates, follow these steps to ensure uninterrupted service:
///
/// 1. **Before Rotation**: Add the new certificate's public key hash to `pinnedPublicKeyHashes`
///    as a backup pin while keeping the current primary pin.
///
/// 2. **Deploy App Update**: Release an app update with both the old (primary) and new (backup)
///    certificate hashes. Wait for sufficient user adoption of this update.
///
/// 3. **Rotate Server Certificate**: Once most users have the updated app, rotate the server
///    certificate. The backup pin will now validate successfully.
///
/// 4. **Update Primary Pin**: In the next app release, move the new hash to the primary position
///    and add a new backup hash for the next rotation cycle.
///
/// **Important**: Always maintain at least two pins (primary + backup) to prevent lockout
/// during certificate rotation. The backup pin should be for a certificate that is ready
/// to be deployed but not yet active on the server.
///
/// ## Generating Public Key Hashes
///
/// To generate a SHA-256 hash of a certificate's public key:
/// ```bash
/// # Extract public key and generate hash
/// openssl s_client -connect api.offleash.app:443 -servername api.offleash.app 2>/dev/null </dev/null \
///   | openssl x509 -pubkey -noout \
///   | openssl pkey -pubin -outform DER \
///   | openssl dgst -sha256 -binary \
///   | base64
/// ```
final class CertificatePinningDelegate: NSObject, URLSessionDelegate {

    /// Pinned public key hashes for certificate validation.
    /// The first hash is the primary (current) certificate, subsequent hashes are backups for rotation.
    /// These are Base64-encoded SHA-256 hashes of the Subject Public Key Info (SPKI).
    static let pinnedPublicKeyHashes: [String] = [
        // Primary certificate hash for api.offleash.app
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        // Backup certificate hash for rotation
        "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB="
    ]

    func urlSession(
        _ session: URLSession,
        didReceive challenge: URLAuthenticationChallenge,
        completionHandler: @escaping (URLSession.AuthChallengeDisposition, URLCredential?) -> Void
    ) {
        guard challenge.protectionSpace.authenticationMethod == NSURLAuthenticationMethodServerTrust,
              let serverTrust = challenge.protectionSpace.serverTrust else {
            completionHandler(.performDefaultHandling, nil)
            return
        }

        // Validate the certificate chain
        if validateCertificate(serverTrust: serverTrust) {
            let credential = URLCredential(trust: serverTrust)
            completionHandler(.useCredential, credential)
        } else {
            completionHandler(.cancelAuthenticationChallenge, nil)
        }
    }

    private func validateCertificate(serverTrust: SecTrust) -> Bool {
        // Get the server's certificate chain
        guard let certificateChain = SecTrustCopyCertificateChain(serverTrust) as? [SecCertificate],
              let serverCertificate = certificateChain.first else {
            return false
        }

        // Extract the public key from the certificate
        guard let publicKey = SecCertificateCopyKey(serverCertificate) else {
            return false
        }

        // Get the public key data in external representation
        var error: Unmanaged<CFError>?
        guard let publicKeyData = SecKeyCopyExternalRepresentation(publicKey, &error) as Data? else {
            return false
        }

        // Determine key type and get appropriate SPKI header
        guard let spkiHeader = spkiHeader(for: publicKey) else {
            return false
        }

        var spkiData = Data(spkiHeader)
        spkiData.append(publicKeyData)

        // Calculate SHA-256 hash of the SPKI
        let hash = sha256(data: spkiData)
        let hashBase64 = hash.base64EncodedString()

        // Check if the hash matches any of our pinned hashes
        return Self.pinnedPublicKeyHashes.contains(hashBase64)
    }

    /// Returns the appropriate SPKI header bytes for the given public key type.
    /// The header is prepended to the raw public key data to form a standard SPKI structure.
    private func spkiHeader(for publicKey: SecKey) -> [UInt8]? {
        guard let attributes = SecKeyCopyAttributes(publicKey) as? [String: Any],
              let keyType = attributes[kSecAttrKeyType as String] as? String,
              let keySize = attributes[kSecAttrKeySizeInBits as String] as? Int else {
            return nil
        }

        // RSA SPKI headers (ASN.1 DER encoded)
        if keyType == (kSecAttrKeyTypeRSA as String) {
            switch keySize {
            case 2048:
                return [
                    0x30, 0x82, 0x01, 0x22, 0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86,
                    0xf7, 0x0d, 0x01, 0x01, 0x01, 0x05, 0x00, 0x03, 0x82, 0x01, 0x0f, 0x00
                ]
            case 4096:
                return [
                    0x30, 0x82, 0x02, 0x22, 0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86,
                    0xf7, 0x0d, 0x01, 0x01, 0x01, 0x05, 0x00, 0x03, 0x82, 0x02, 0x0f, 0x00
                ]
            default:
                return nil
            }
        }

        // EC SPKI headers (ASN.1 DER encoded)
        if keyType == (kSecAttrKeyTypeECSECPrimeRandom as String) {
            switch keySize {
            case 256:
                // secp256r1 / P-256
                return [
                    0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02,
                    0x01, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, 0x03,
                    0x42, 0x00
                ]
            case 384:
                // secp384r1 / P-384
                return [
                    0x30, 0x76, 0x30, 0x10, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02,
                    0x01, 0x06, 0x05, 0x2b, 0x81, 0x04, 0x00, 0x22, 0x03, 0x62, 0x00
                ]
            default:
                return nil
            }
        }

        return nil
    }

    private func sha256(data: Data) -> Data {
        var hash = [UInt8](repeating: 0, count: Int(CC_SHA256_DIGEST_LENGTH))
        data.withUnsafeBytes { buffer in
            _ = CC_SHA256(buffer.baseAddress, CC_LONG(data.count), &hash)
        }
        return Data(hash)
    }
}

// MARK: - API Client

actor APIClient {
    static let shared = APIClient()

    private let baseURL: String
    private let session: URLSession
    private let decoder: JSONDecoder
    private let encoder: JSONEncoder
    private let certificatePinningDelegate: CertificatePinningDelegate

    private init() {
        // Configure base URL from environment or use default
        // Note: iOS Simulator can't use localhost - use Mac's IP address
        #if DEBUG
        self.baseURL = ProcessInfo.processInfo.environment["API_BASE_URL"] ?? "http://192.168.25.201:8080"
        #else
        self.baseURL = ProcessInfo.processInfo.environment["API_BASE_URL"] ?? "https://api.offleash.app"
        #endif

        // Configure URLSession with certificate pinning delegate
        let configuration = URLSessionConfiguration.default
        configuration.timeoutIntervalForRequest = 30
        configuration.timeoutIntervalForResource = 60
        self.certificatePinningDelegate = CertificatePinningDelegate()

        // Skip certificate pinning for localhost/development
        let isLocalhost = self.baseURL.contains("localhost") || self.baseURL.contains("127.0.0.1") || self.baseURL.contains("192.168.")
        self.session = URLSession(
            configuration: configuration,
            delegate: isLocalhost ? nil : certificatePinningDelegate,
            delegateQueue: nil
        )

        // Configure JSON decoder
        self.decoder = JSONDecoder()
        self.decoder.keyDecodingStrategy = .convertFromSnakeCase
        self.decoder.dateDecodingStrategy = .iso8601

        // Configure JSON encoder
        self.encoder = JSONEncoder()
        self.encoder.keyEncodingStrategy = .convertToSnakeCase
        self.encoder.dateEncodingStrategy = .iso8601
    }

    // MARK: - Public Methods

    /// Perform a GET request
    func get<T: Decodable>(_ path: String, queryItems: [URLQueryItem]? = nil) async throws -> T {
        try await request(path: path, method: .get, queryItems: queryItems)
    }

    /// Perform a POST request with a body
    func post<T: Decodable, B: Encodable>(_ path: String, body: B) async throws -> T {
        try await request(path: path, method: .post, body: body)
    }

    /// Perform a POST request without a body
    func post<T: Decodable>(_ path: String) async throws -> T {
        try await request(path: path, method: .post)
    }

    /// Perform a PUT request with a body
    func put<T: Decodable, B: Encodable>(_ path: String, body: B) async throws -> T {
        try await request(path: path, method: .put, body: body)
    }

    /// Perform a PUT request without a body
    func put<T: Decodable>(_ path: String) async throws -> T {
        try await request(path: path, method: .put)
    }

    /// Perform a DELETE request
    func delete<T: Decodable>(_ path: String) async throws -> T {
        try await request(path: path, method: .delete)
    }

    /// Perform a DELETE request that returns no content
    func delete(_ path: String) async throws {
        let _: EmptyResponse = try await request(path: path, method: .delete)
    }

    // MARK: - Token Management

    func setAuthToken(_ token: String) {
        _ = KeychainHelper.shared.saveToken(token)
        NotificationCenter.default.post(
            name: .authStateChanged,
            object: nil,
            userInfo: ["isAuthenticated": true]
        )
    }

    func clearAuthToken() {
        KeychainHelper.shared.deleteToken()
        Task { @MainActor in
            if FirebaseState.isConfigured { Crashlytics.crashlytics().setUserID("") }
            UserSession.shared.clearUser()
        }
        NotificationCenter.default.post(
            name: .authStateChanged,
            object: nil,
            userInfo: ["isAuthenticated": false]
        )
    }

    var isAuthenticated: Bool {
        KeychainHelper.shared.hasToken
    }

    /// Validate the stored authentication token
    func validateToken() async throws -> TokenValidationResponse {
        try await get("/auth/validate")
    }

    // MARK: - Private Methods

    private func request<T: Decodable>(
        path: String,
        method: HTTPMethod,
        queryItems: [URLQueryItem]? = nil,
        body: (any Encodable)? = nil
    ) async throws -> T {
        // Build URL
        guard var urlComponents = URLComponents(string: baseURL + path) else {
            let error = APIError.invalidURL
            trackAPIError(error, path: path, method: method, statusCode: nil)
            throw error
        }

        if let queryItems = queryItems, !queryItems.isEmpty {
            urlComponents.queryItems = queryItems
        }

        guard let url = urlComponents.url else {
            let error = APIError.invalidURL
            trackAPIError(error, path: path, method: method, statusCode: nil)
            throw error
        }

        // Build request
        var request = URLRequest(url: url)
        request.httpMethod = method.rawValue
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("application/json", forHTTPHeaderField: "Accept")

        // Add authorization header if token exists
        if let token = KeychainHelper.shared.getToken() {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }

        // Encode body if present
        if let body = body {
            do {
                request.httpBody = try encoder.encode(AnyEncodable(body))
            } catch {
                let apiError = APIError.encodingError(error)
                trackAPIError(apiError, path: path, method: method, statusCode: nil)
                throw apiError
            }
        }

        // Perform request
        let data: Data
        let response: URLResponse

        do {
            (data, response) = try await session.data(for: request)
        } catch {
            let apiError = APIError.networkError(error)
            trackAPIError(apiError, path: path, method: method, statusCode: nil)
            throw apiError
        }

        // Validate response
        guard let httpResponse = response as? HTTPURLResponse else {
            let error = APIError.invalidResponse
            trackAPIError(error, path: path, method: method, statusCode: nil)
            throw error
        }

        // Handle HTTP status codes
        switch httpResponse.statusCode {
        case 200...299:
            break
        case 401:
            // Clear token on unauthorized
            KeychainHelper.shared.deleteToken()
            Task { @MainActor in
                if FirebaseState.isConfigured { Crashlytics.crashlytics().setUserID("") }
            }
            NotificationCenter.default.post(
                name: .authStateChanged,
                object: nil,
                userInfo: ["isAuthenticated": false]
            )
            let error = APIError.unauthorized
            trackAPIError(error, path: path, method: method, statusCode: httpResponse.statusCode)
            throw error
        default:
            // Try to extract error message from response
            let errorMessage = try? decoder.decode(ErrorResponse.self, from: data).message
            let error = APIError.httpError(statusCode: httpResponse.statusCode, message: errorMessage)
            trackAPIError(error, path: path, method: method, statusCode: httpResponse.statusCode)
            throw error
        }

        // Handle empty response for void return types
        if T.self == EmptyResponse.self {
            return EmptyResponse() as! T
        }

        // Decode response
        do {
            return try decoder.decode(T.self, from: data)
        } catch {
            let apiError = APIError.decodingError(error)
            trackAPIError(apiError, path: path, method: method, statusCode: httpResponse.statusCode)
            throw apiError
        }
    }

    private func trackAPIError(_ error: APIError, path: String, method: HTTPMethod, statusCode: Int?) -> Void {
        let errorType = errorTypeName(for: error)
        var context = "\(method.rawValue) \(path)"
        if let statusCode = statusCode {
            context += " [\(statusCode)]"
        }
        context += " - \(errorType)"

        Task { @MainActor in
            StubAnalyticsService.shared.trackError(error: error, context: context)
        }
    }

    private func errorTypeName(for error: APIError) -> String {
        switch error {
        case .invalidURL:
            return "invalidURL"
        case .invalidResponse:
            return "invalidResponse"
        case .httpError:
            return "httpError"
        case .decodingError:
            return "decodingError"
        case .encodingError:
            return "encodingError"
        case .networkError:
            return "networkError"
        case .unauthorized:
            return "unauthorized"
        case .noData:
            return "noData"
        case .certificatePinningFailed:
            return "certificatePinningFailed"
        }
    }
}

// MARK: - Helper Types

/// Empty response for endpoints that return no content
struct EmptyResponse: Decodable {}

/// Error response structure
private struct ErrorResponse: Decodable {
    let message: String?
    let error: String?
}

/// Type-erased encodable wrapper
private struct AnyEncodable: Encodable {
    private let encode: (Encoder) throws -> Void

    init<T: Encodable>(_ value: T) {
        self.encode = { encoder in
            try value.encode(to: encoder)
        }
    }

    func encode(to encoder: Encoder) throws {
        try encode(encoder)
    }
}

// MARK: - Convenience Extensions

extension APIClient {
    /// Configure the API client with a custom base URL (useful for testing or different environments)
    func configure(baseURL: String) -> APIClient {
        // Note: Since APIClient uses actor isolation, we'd need to recreate it
        // For now, base URL is configured via environment variable
        return self
    }
}
