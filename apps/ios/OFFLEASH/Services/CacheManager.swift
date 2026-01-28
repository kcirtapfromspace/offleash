import Foundation

// MARK: - Cache Entry

/// Wrapper that stores cached data with expiry metadata
private struct CacheEntry<T: Codable>: Codable {
    let value: T
    let expiryDate: Date

    var isExpired: Bool {
        Date() > expiryDate
    }
}

// MARK: - Cache Metadata

/// Metadata for tracking cache entry sizes and access times for LRU eviction
private struct CacheMetadata: Codable {
    var entries: [String: EntryMetadata]

    struct EntryMetadata: Codable {
        let size: Int
        var lastAccessTime: Date
    }

    init() {
        self.entries = [:]
    }
}

// MARK: - Cache Manager

/// Thread-safe cache manager that persists data to disk with TTL support
actor CacheManager {
    static let shared = CacheManager()

    /// Maximum cache size in bytes (default: 50MB)
    let maxCacheSize: Int

    private let fileManager: FileManager
    private let cacheDirectory: URL
    private let encoder: JSONEncoder
    private let decoder: JSONDecoder
    private let metadataFileURL: URL
    private var metadata: CacheMetadata

    private init(maxCacheSize: Int = 50 * 1024 * 1024) {
        self.maxCacheSize = maxCacheSize
        self.fileManager = FileManager.default
        self.encoder = JSONEncoder()
        self.decoder = JSONDecoder()

        // Get the caches directory
        let cachesURL = fileManager.urls(for: .cachesDirectory, in: .userDomainMask).first!
        self.cacheDirectory = cachesURL.appendingPathComponent("com.offleash.cache", isDirectory: true)
        self.metadataFileURL = cacheDirectory.appendingPathComponent(".metadata")

        // Create cache directory if it doesn't exist
        try? fileManager.createDirectory(at: cacheDirectory, withIntermediateDirectories: true)

        // Load existing metadata or create new
        if let data = try? Data(contentsOf: metadataFileURL),
           let loadedMetadata = try? decoder.decode(CacheMetadata.self, from: data) {
            self.metadata = loadedMetadata
        } else {
            self.metadata = CacheMetadata()
        }
    }

    // MARK: - Public Methods

    /// Retrieve a cached value for the given key
    /// Returns nil if the key doesn't exist or the TTL has expired
    func get<T: Codable>(key: String) -> T? {
        let fileURL = cacheFileURL(for: key)

        guard fileManager.fileExists(atPath: fileURL.path) else {
            return nil
        }

        do {
            let data = try Data(contentsOf: fileURL)
            let entry = try decoder.decode(CacheEntry<T>.self, from: data)

            // Check if expired
            if entry.isExpired {
                // Remove expired entry
                removeEntryAndMetadata(key: key, fileURL: fileURL)
                return nil
            }

            // Update last access time for LRU tracking
            updateAccessTime(for: key)

            return entry.value
        } catch {
            // Remove corrupted cache file
            removeEntryAndMetadata(key: key, fileURL: fileURL)
            return nil
        }
    }

    /// Store a value in the cache with the given key and TTL
    /// - Parameters:
    ///   - key: The cache key
    ///   - value: The value to cache (must be Codable)
    ///   - ttl: Time-to-live in seconds (default: 1 hour)
    func set<T: Codable>(key: String, value: T, ttl: TimeInterval = 3600) {
        let expiryDate = Date().addingTimeInterval(ttl)
        let entry = CacheEntry(value: value, expiryDate: expiryDate)

        do {
            let data = try encoder.encode(entry)
            let fileURL = cacheFileURL(for: key)

            // Remove old entry size from tracking if it exists
            let oldSize = metadata.entries[key]?.size ?? 0

            // Calculate new total size
            let newSize = data.count
            let currentTotal = currentCacheSize()
            let projectedTotal = currentTotal - oldSize + newSize

            // Evict LRU entries if we exceed the limit
            if projectedTotal > maxCacheSize {
                evictLRUEntries(targetSize: projectedTotal - maxCacheSize)
            }

            // Write the data
            try data.write(to: fileURL, options: .atomic)

            // Update metadata
            metadata.entries[key] = CacheMetadata.EntryMetadata(
                size: newSize,
                lastAccessTime: Date()
            )
            saveMetadata()
        } catch {
            // Silently fail - caching is best-effort
        }
    }

    /// Remove a cached value for the given key
    func remove(key: String) {
        let fileURL = cacheFileURL(for: key)
        removeEntryAndMetadata(key: key, fileURL: fileURL)
    }

    /// Invalidate a cached value for the given key
    /// Use this to clear stale data when the underlying data has been modified
    func invalidate(key: String) {
        remove(key: key)
    }

    /// Clear all cached data
    func clear() {
        try? fileManager.removeItem(at: cacheDirectory)
        try? fileManager.createDirectory(at: cacheDirectory, withIntermediateDirectories: true)
        metadata = CacheMetadata()
    }

    /// Get the current total cache size in bytes
    /// Useful for debugging and monitoring cache usage
    func currentCacheSize() -> Int {
        metadata.entries.values.reduce(0) { $0 + $1.size }
    }

    // MARK: - Private Methods

    /// Generate a file URL for a cache key
    private func cacheFileURL(for key: String) -> URL {
        // Use a hash of the key to create a valid filename
        let safeKey = key.data(using: .utf8)?.base64EncodedString() ?? key
        let sanitizedKey = safeKey.replacingOccurrences(of: "/", with: "_")
        return cacheDirectory.appendingPathComponent(sanitizedKey)
    }

    /// Update the last access time for a cache entry
    private func updateAccessTime(for key: String) {
        guard var entryMetadata = metadata.entries[key] else { return }
        entryMetadata.lastAccessTime = Date()
        metadata.entries[key] = entryMetadata
        saveMetadata()
    }

    /// Remove a cache entry and its metadata
    private func removeEntryAndMetadata(key: String, fileURL: URL) {
        try? fileManager.removeItem(at: fileURL)
        metadata.entries.removeValue(forKey: key)
        saveMetadata()
    }

    /// Evict least recently used entries until we free up at least targetSize bytes
    private func evictLRUEntries(targetSize: Int) {
        var freedSpace = 0

        // Sort entries by last access time (oldest first)
        let sortedEntries = metadata.entries.sorted { $0.value.lastAccessTime < $1.value.lastAccessTime }

        for (key, entryMetadata) in sortedEntries {
            if freedSpace >= targetSize {
                break
            }

            let fileURL = cacheFileURL(for: key)
            try? fileManager.removeItem(at: fileURL)
            metadata.entries.removeValue(forKey: key)
            freedSpace += entryMetadata.size
        }

        saveMetadata()
    }

    /// Persist metadata to disk
    private func saveMetadata() {
        if let data = try? encoder.encode(metadata) {
            try? data.write(to: metadataFileURL, options: .atomic)
        }
    }
}
