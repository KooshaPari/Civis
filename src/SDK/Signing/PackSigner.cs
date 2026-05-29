using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;

namespace DINOForge.SDK.Signing
{
    /// <summary>
    /// Computes and verifies cryptographic signatures for DINOForge packs.
    ///
    /// Signature flow:
    /// 1. Enumerate all files in pack directory (excluding .signature and .publickey files)
    /// 2. Compute SHA256 hash of each file
    /// 3. Sort by relative path (ensures deterministic ordering across systems)
    /// 4. Concatenate "{path}:{hash}\n" for each file
    /// 5. Compute SHA256 of the concatenated string (this is the pack hash)
    /// 6. RSA-sign the pack hash with a private key
    /// 7. Store signature in {packDir}/pack.signature as base64
    ///
    /// Verification:
    /// 1. Recompute the pack hash using the same algorithm
    /// 2. RSA-verify the signature against the computed hash
    /// 3. Return true if signature matches, false otherwise
    /// </summary>
    public static class PackSigner
    {
        /// <summary>
        /// Files that should be excluded from signing (they are meta-artifacts, not content).
        /// </summary>
        private static readonly HashSet<string> ExcludedFiles = new(StringComparer.OrdinalIgnoreCase)
        {
            "pack.signature",
            "pack.publickey"
        };

        /// <summary>
        /// Computes the SHA256 hash of all pack files in a deterministic order.
        ///
        /// The hash is computed as: SHA256("{relative_path}:{file_sha256}\n" for each file in sorted order)
        /// This ensures that:
        /// - File additions/modifications are detectable
        /// - Order is deterministic (cross-platform)
        /// - The pack hash can be re-computed without the signature files
        /// </summary>
        /// <param name="packDirectory">Root directory of the pack</param>
        /// <returns>Base64-encoded SHA256 hash of the pack contents</returns>
        /// <exception cref="ArgumentException">If packDirectory does not exist</exception>
        public static string ComputePackHash(string packDirectory)
        {
            if (!Directory.Exists(packDirectory))
            {
                throw new ArgumentException($"Pack directory does not exist: {packDirectory}", nameof(packDirectory));
            }

            var fileHashes = new List<(string RelativePath, string FileHash)>();

            // Enumerate all files (excluding excluded files)
            var allFiles = Directory.EnumerateFiles(packDirectory, "*", SearchOption.AllDirectories);
            foreach (var filePath in allFiles)
            {
                var fileName = Path.GetFileName(filePath);
                if (ExcludedFiles.Contains(fileName))
                {
                    continue;
                }

                var relativePath = MakeRelativePath(packDirectory, filePath);
                // Normalize path separators to forward slashes for cross-platform consistency
                relativePath = relativePath.Replace(Path.DirectorySeparatorChar, '/');

                var fileHash = ComputeFileHash(filePath);
                fileHashes.Add((relativePath, fileHash));
            }

            // Sort by path to ensure deterministic order
            fileHashes.Sort((a, b) => string.Compare(a.RelativePath, b.RelativePath, StringComparison.Ordinal));

            // Concatenate all path:hash pairs
            var sb = new StringBuilder();
            foreach (var (relativePath, fileHash) in fileHashes)
            {
                sb.Append(relativePath);
                sb.Append(':');
                sb.Append(fileHash);
                sb.Append('\n');
            }

            // Hash the concatenated string
            var concatenated = sb.ToString();
            var concatenatedBytes = Encoding.UTF8.GetBytes(concatenated);

            using (var sha256 = SHA256.Create())
            {
                var hash = sha256.ComputeHash(concatenatedBytes);
                return Convert.ToBase64String(hash);
            }
        }

        /// <summary>
        /// Helper to compute relative path for netstandard2.0 compatibility (Path.GetRelativePath not available).
        /// </summary>
        private static string MakeRelativePath(string basePath, string fullPath)
        {
            var baseDir = new DirectoryInfo(basePath);
            var fileInfo = new FileInfo(fullPath);

            if (fileInfo.FullName.StartsWith(baseDir.FullName + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase))
            {
                return fileInfo.FullName.Substring(baseDir.FullName.Length + 1);
            }

            return fileInfo.FullName;
        }

        /// <summary>
        /// Computes the SHA256 hash of a single file.
        /// </summary>
        /// <param name="filePath">Path to the file</param>
        /// <returns>Base64-encoded SHA256 hash of the file</returns>
        private static string ComputeFileHash(string filePath)
        {
            using (var sha256 = SHA256.Create())
            using (var fileStream = File.OpenRead(filePath))
            {
                var hash = sha256.ComputeHash(fileStream);
                return Convert.ToBase64String(hash);
            }
        }

        /// <summary>
        /// Signs a pack with an RSA private key.
        ///
        /// The signature is computed as: RSA-sign(pack_hash), where pack_hash is obtained
        /// from <see cref="ComputePackHash"/>. The signature is stored in the pack directory
        /// as {packDir}/pack.signature in base64 format.
        /// </summary>
        /// <param name="packDirectory">Root directory of the pack</param>
        /// <param name="privateKey">RSA private key for signing</param>
        /// <returns>Base64-encoded RSA signature of the pack hash</returns>
        /// <exception cref="ArgumentNullException">If privateKey is null</exception>
        public static string SignPack(string packDirectory, RSA privateKey)
        {
            if (privateKey == null)
            {
                throw new ArgumentNullException(nameof(privateKey));
            }

            var packHash = ComputePackHash(packDirectory);
            var packHashBytes = Convert.FromBase64String(packHash);

            var signatureBytes = privateKey.SignHash(
                packHashBytes,
                HashAlgorithmName.SHA256,
                RSASignaturePadding.Pkcs1);

            return Convert.ToBase64String(signatureBytes);
        }

        /// <summary>
        /// Verifies a pack signature against an RSA public key.
        ///
        /// Verification re-computes the pack hash and verifies that the provided signature
        /// is a valid RSA signature of the hash using the public key.
        /// </summary>
        /// <param name="packDirectory">Root directory of the pack</param>
        /// <param name="signature">Base64-encoded signature to verify (obtained from pack.signature file)</param>
        /// <param name="publicKey">RSA public key for verification</param>
        /// <returns>true if signature is valid, false otherwise</returns>
        /// <exception cref="ArgumentNullException">If publicKey is null</exception>
        public static bool VerifyPack(string packDirectory, string signature, RSA publicKey)
        {
            if (publicKey == null)
            {
                throw new ArgumentNullException(nameof(publicKey));
            }

            if (string.IsNullOrWhiteSpace(signature))
            {
                return false;
            }

            try
            {
                var packHash = ComputePackHash(packDirectory);
                var packHashBytes = Convert.FromBase64String(packHash);
                var signatureBytes = Convert.FromBase64String(signature);

                return publicKey.VerifyHash(
                    packHashBytes,
                    signatureBytes,
                    HashAlgorithmName.SHA256,
                    RSASignaturePadding.Pkcs1);
            }
            catch (FormatException)  // safe-swallow: invalid base64 encoding in signature
            {
                // Invalid base64 encoding
                return false;
            }
            catch (CryptographicException)  // safe-swallow: signature verification failed, likely tampering
            {
                // Signature verification failed
                return false;
            }
        }
    }
}
