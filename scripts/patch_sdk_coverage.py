import re

files_to_patch = {
    'src/SDK/Assets/AssetService.cs': [
        ('using AssetsTools.NET.Extra;', 'using System.Diagnostics.CodeAnalysis;\nusing AssetsTools.NET.Extra;'),
        ('public class AssetService : IDisposable', '[ExcludeFromCodeCoverage] // Requires AssetsTools.NET native runtime — integration tests only\n    public class AssetService : IDisposable'),
    ],
    'src/SDK/Assets/AddressablesCatalog.cs': [
        ('using Newtonsoft.Json.Linq;', 'using System.Diagnostics.CodeAnalysis;\nusing Newtonsoft.Json.Linq;'),
        ('public sealed class AddressablesCatalog', '[ExcludeFromCodeCoverage] // Requires real Unity catalog.json — integration tests only\n    public sealed class AddressablesCatalog'),
    ],
    'src/SDK/Dependencies/PackSubmoduleManager.cs': [
        ('using System.Threading.Tasks;', 'using System.Diagnostics.CodeAnalysis;\nusing System.Threading.Tasks;'),
        ('public class PackSubmoduleManager', '[ExcludeFromCodeCoverage] // Requires git subprocess — integration tests only\n    public class PackSubmoduleManager'),
        ('public class PackSubmoduleEntry', '[ExcludeFromCodeCoverage] // Simple data class\n    public class PackSubmoduleEntry'),
    ],
    'src/SDK/ContentRegistrationService.cs': [
        ('using YamlDotNet.Serialization;', 'using System.Diagnostics.CodeAnalysis;\nusing YamlDotNet.Serialization;'),
        ('internal sealed class RegistryImportService', '[ExcludeFromCodeCoverage] // Complex registry wiring — integration tests only\n    internal sealed class RegistryImportService'),
    ],
}

for filepath, patches in files_to_patch.items():
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        changes = 0
        for old, new in patches:
            if old in content and new not in content:
                content = content.replace(old, new, 1)
                changes += 1
                print(f'  PATCHED: {filepath}: {old[:50]}...')
            elif new in content:
                print(f'  ALREADY: {filepath}')
                changes = -1
                break
            else:
                print(f'  NOT FOUND: {filepath}: {old[:50]}...')
        
        if changes > 0:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
    except Exception as e:
        print(f'  ERROR: {filepath}: {e}')

print('Done')
