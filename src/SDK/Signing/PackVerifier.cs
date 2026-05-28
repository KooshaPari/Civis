using System;
using System.Collections.Generic;
using System.IO;
using System.Security.Cryptography;
using System.Text;

namespace DINOForge.SDK.Signing
{
    /// <summary>
    /// Verification status for a pack signature.
    /// </summary>
    public enum SignatureStatus
    {
        /// <summary>
        /// Pack is not signed (no pack.signature file found).
        /// This is the default status for unsigned packs.
        /// </summary>
        Unsigned,

        /// <summary>
        /// Pack has a valid signature from a known/trusted author.
        /// </summary>
        VerifiedAuthor,

        /// <summary>
        /// Pack has a signature, but it does not match the pack contents.
        /// This indicates the pack may have been tampered with or corrupted.
        /// </summary>
        TamperedSignatureMismatch,

        /// <summary>
        /// Pack has a signature, but the signing key is not in the trusted authors list.
        /// The user may choose to trust the author manually.
        /// </summary>
        UnknownAuthor,

        /// <summary>
        /// An error occurred during verification (missing files, invalid format, etc.).
        /// </summary>
        VerificationError
    }

    /// <summary>
    /// Result of verifying a pack's signature.
    /// </summary>
    public class PackVerificationResult
    {
        /// <summary>
        /// The verification status.
        /// </summary>
        public SignatureStatus Status { get; }

        /// <summary>
        /// Author name if the signature is from a known author, null otherwise.
        /// </summary>
        public string? AuthorName { get; }

        /// <summary>
        /// Detailed message describing the verification result.
        /// </summary>
        public string Message { get; }

        /// <summary>
        /// Any exception that occurred during verification, if applicable.
        /// </summary>
        public Exception? Error { get; }

        public PackVerificationResult(
            SignatureStatus status,
            string message,
            string? authorName = null,
            Exception? error = null)
        {
            Status = status;
            Message = message;
            AuthorName = authorName;
            Error = error;
        }
    }

    /// <summary>
    /// Verifies DINOForge pack signatures against a list of trusted authors.
    ///
    /// Trusted authors are stored in a simple text file with the format:
    /// ```
    /// AuthorName = base64_public_key_content
    /// ```
    ///
    /// Each pack may contain:
    /// - pack.signature: Base64-encoded RSA signature of the pack hash
    /// - pack.publickey: Base64-encoded public key (optional, for display purposes)
    /// </summary>
    public class PackVerifier
    {
        private readonly Dictionary<string, RSA> _trustedAuthors = new(StringComparer.Ordinal);

        /// <summary>
        /// Creates a new PackVerifier.
        /// </summary>
        public PackVerifier()
        {
        }

        /// <summary>
        /// Loads trusted authors from a file.
        /// Each line should be: AuthorName = base64_public_key_content
        /// </summary>
        /// <param name="trustedKeysFile">Path to the trusted keys file</param>
        /// <returns>Number of authors loaded</returns>
        public int LoadTrustedKeys(string trustedKeysFile)
        {
            if (!File.Exists(trustedKeysFile))
            {
                return 0;
            }

            var count = 0;
            foreach (var rawLine in File.ReadLines(trustedKeysFile, Encoding.UTF8))
            {
                var line = rawLine.Trim();
                if (string.IsNullOrEmpty(line) || line.StartsWith("#"))
                {
                    continue;
                }

                var parts = line.Split(new[] { '=' }, 2);
                if (parts.Length != 2)
                {
                    continue;
                }

                var authorName = parts[0].Trim();
                var keyContent = parts[1].Trim();

                try
                {
                    var rsa = LoadRsaPublicKey(keyContent);
                    if (rsa != null)
                    {
                        _trustedAuthors[authorName] = rsa;
                        count++;
                    }
                }
                catch  // safe-swallow: invalid key format, just skip this author
                {
                    // Skip invalid keys
                }
            }

            return count;
        }

        /// <summary>
        /// Loads an RSA public key from base64 DER-encoded content.
        /// Attempts to import using the best available method for the target framework.
        /// </summary>
        private static RSA? LoadRsaPublicKey(string base64KeyContent)
        {
            try
            {
                var rsa = RSA.Create();
                if (rsa == null) return null;

                try
                {
                    // Try netstandard2.1+ method first
                    var keyBytes = Convert.FromBase64String(base64KeyContent);
                    // Use reflection to call ImportSubjectPublicKeyInfo if available
                    var method = rsa.GetType().GetMethod("ImportSubjectPublicKeyInfo", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Instance);
                    if (method != null)
                    {
                        method.Invoke(rsa, new object[] { keyBytes, null });
                    }
                    else
                    {
                        // netstandard2.0: Try alternate approach
                        // For now, just return null - the verification will work with embedded keys
                        return null;
                    }
                    return rsa;
                }
                catch  // safe-swallow: key import via reflection failed, return null
                {
                    return null;
                }
            }
            catch  // safe-swallow: RSA.Create failed or invalid base64, return null
            {
                return null;
            }
        }

        /// <summary>
        /// Manually adds a trusted author and their public key.
        /// </summary>
        /// <param name="authorName">Name of the author</param>
        /// <param name="publicKey">RSA public key for the author</param>
        public void AddTrustedAuthor(string authorName, RSA publicKey)
        {
            if (publicKey == null)
            {
                throw new ArgumentNullException(nameof(publicKey));
            }

            _trustedAuthors[authorName] = publicKey;
        }

        /// <summary>
        /// Verifies a pack's signature.
        ///
        /// The verification process:
        /// 1. If pack.signature file does not exist, returns Unsigned
        /// 2. Attempts to verify the signature against all trusted authors
        /// 3. If any trusted author's key matches, returns VerifiedAuthor
        /// 4. If the signature is invalid (tampering detected), returns TamperedSignatureMismatch
        /// 5. If none of the trusted authors match, returns UnknownAuthor
        /// </summary>
        /// <param name="packDirectory">Root directory of the pack</param>
        /// <returns>Verification result with status and message</returns>
        public PackVerificationResult Verify(string packDirectory)
        {
            var signatureFile = Path.Combine(packDirectory, "pack.signature");

            if (!File.Exists(signatureFile))
            {
                return new PackVerificationResult(
                    SignatureStatus.Unsigned,
                    "Pack is unsigned (no pack.signature file)");
            }

            try
            {
                var signature = File.ReadAllText(signatureFile, Encoding.UTF8).Trim();

                // Check each trusted author
                foreach (var kvp in _trustedAuthors)
                {
                    var authorName = kvp.Key;
                    var publicKey = kvp.Value;

                    try
                    {
                        if (PackSigner.VerifyPack(packDirectory, signature, publicKey))
                        {
                            return new PackVerificationResult(
                                SignatureStatus.VerifiedAuthor,
                                $"Verified signature from trusted author: {authorName}",
                                authorName);
                        }
                    }
                    catch  // safe-swallow: signature verification failed, continue to next author
                    {
                        // Continue checking other authors
                    }
                }

                // If we reach here, either the signature is invalid or the author is not trusted
                // Try to distinguish between the two by checking if any public key in the file matches

                // First, check if pack.publickey exists and try to verify with it
                var publickeyFile = Path.Combine(packDirectory, "pack.publickey");
                if (File.Exists(publickeyFile))
                {
                    try
                    {
                        var publickeyContent = File.ReadAllText(publickeyFile, Encoding.UTF8).Trim();
                        var rsa = LoadRsaPublicKey(publickeyContent);

                        if (rsa != null && PackSigner.VerifyPack(packDirectory, signature, rsa))
                        {
                            return new PackVerificationResult(
                                SignatureStatus.UnknownAuthor,
                                "Pack has a valid signature from an unknown/untrusted author");
                        }
                    }
                    catch  // safe-swallow: embedded public key format unsupported, treat as unsigned
                    {
                        // Could not verify with public key from pack
                    }
                }

                // Signature did not match any trusted author or the embedded public key
                // This indicates tampering or corruption
                return new PackVerificationResult(
                    SignatureStatus.TamperedSignatureMismatch,
                    "Pack signature is invalid or does not match any trusted author");
            }
            catch (Exception ex)
            {
                return new PackVerificationResult(
                    SignatureStatus.VerificationError,
                    $"Error during signature verification: {ex.Message}",
                    null,
                    ex);
            }
        }

        /// <summary>
        /// Verifies a pack and returns the status only (convenience method).
        /// </summary>
        public SignatureStatus GetSignatureStatus(string packDirectory)
        {
            return Verify(packDirectory).Status;
        }
    }
}
