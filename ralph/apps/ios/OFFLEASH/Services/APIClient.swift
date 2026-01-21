import Foundation
import Security

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

final class KeychainHelper {
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

// MARK: - API Client

actor APIClient {
    static let shared = APIClient()

    private let baseURL: String
    private let session: URLSession
    private let decoder: JSONDecoder
    private let encoder: JSONEncoder

    private init() {
        // Configure base URL from environment or use default
        self.baseURL = ProcessInfo.processInfo.environment["API_BASE_URL"] ?? "https://api.offleash.app"

        // Configure URLSession
        let configuration = URLSessionConfiguration.default
        configuration.timeoutIntervalForRequest = 30
        configuration.timeoutIntervalForResource = 60
        self.session = URLSession(configuration: configuration)

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
        NotificationCenter.default.post(
            name: .authStateChanged,
            object: nil,
            userInfo: ["isAuthenticated": false]
        )
    }

    var isAuthenticated: Bool {
        KeychainHelper.shared.hasToken
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
            throw APIError.invalidURL
        }

        if let queryItems = queryItems, !queryItems.isEmpty {
            urlComponents.queryItems = queryItems
        }

        guard let url = urlComponents.url else {
            throw APIError.invalidURL
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
                throw APIError.encodingError(error)
            }
        }

        // Perform request
        let data: Data
        let response: URLResponse

        do {
            (data, response) = try await session.data(for: request)
        } catch {
            throw APIError.networkError(error)
        }

        // Validate response
        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }

        // Handle HTTP status codes
        switch httpResponse.statusCode {
        case 200...299:
            break
        case 401:
            // Clear token on unauthorized
            KeychainHelper.shared.deleteToken()
            NotificationCenter.default.post(
                name: .authStateChanged,
                object: nil,
                userInfo: ["isAuthenticated": false]
            )
            throw APIError.unauthorized
        default:
            // Try to extract error message from response
            let errorMessage = try? decoder.decode(ErrorResponse.self, from: data).message
            throw APIError.httpError(statusCode: httpResponse.statusCode, message: errorMessage)
        }

        // Handle empty response for void return types
        if T.self == EmptyResponse.self {
            return EmptyResponse() as! T
        }

        // Decode response
        do {
            return try decoder.decode(T.self, from: data)
        } catch {
            throw APIError.decodingError(error)
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
