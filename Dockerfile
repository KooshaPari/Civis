FROM mcr.microsoft.com/dotnet/sdk:8.0-alpine

WORKDIR /workspace

# Copy entire repository
COPY . .

# Clean and build
RUN dotnet clean src/DINOForge.sln || true
RUN dotnet build src/Tools/PackCompiler/DINOForge.Tools.PackCompiler.csproj -c Debug

# Run tests to verify
RUN dotnet test src/Tools/PackCompiler/DINOForge.Tools.PackCompiler.csproj --verbosity quiet

# Default entrypoint for asset pipeline
ENTRYPOINT ["dotnet", "run", "--no-build", "--project", "src/Tools/PackCompiler", "--"]
