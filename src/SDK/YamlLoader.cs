#nullable enable
using System.IO;
using DINOForge.SDK.IO;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.SDK
{
    /// <summary>
    /// Centralized YAML deserialization service with consistent settings.
    /// Eliminates duplicated DeserializerBuilder code across the codebase.
    /// </summary>
    public static class YamlLoader
    {
        private static readonly IDeserializer _deserializer;
        private static readonly ISerializer _serializer;

        static YamlLoader()
        {
            _deserializer = new DeserializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .IgnoreUnmatchedProperties()
                .Build();

            _serializer = new SerializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .Build();
        }

        /// <summary>
        /// Deserializes YAML content to the specified type.
        /// </summary>
        public static T? Deserialize<T>(string yaml)
        {
            if (string.IsNullOrEmpty(yaml))
                return default;

            return _deserializer.Deserialize<T>(yaml);
        }

        /// <summary>
        /// Deserializes YAML from a file to the specified type.
        /// </summary>
        public static T? DeserializeFromFile<T>(string filePath)
        {
            if (!File.Exists(filePath))
                return default;

            string yaml = SafeFileIO.ReadText(filePath);
            return Deserialize<T>(yaml);
        }

        /// <summary>
        /// Serializes an object to YAML string.
        /// </summary>
        public static string Serialize<T>(T? obj)
        {
            if (obj == null)
                return string.Empty;

            return _serializer.Serialize(obj);
        }

        /// <summary>
        /// Serializes an object to a YAML file.
        /// </summary>
        public static void SerializeToFile<T>(string filePath, T? obj)
        {
            if (obj == null)
                return;

            string yaml = Serialize(obj);
            SafeFileIO.WriteText(filePath, yaml);
        }

        /// <summary>
        /// Gets the shared deserializer instance for custom operations.
        /// </summary>
        public static IDeserializer Deserializer => _deserializer;

        /// <summary>
        /// Gets the shared serializer instance for custom operations.
        /// </summary>
        public static ISerializer Serializer => _serializer;
    }
}
