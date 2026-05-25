with open('src/Tests/GameClientCoverageTests.cs', 'r') as f:
    content = f.read()

# Fix WithMessage().Contain() to WithMessage("*...*")
content = content.replace('ex.WithMessage().Contain("null result")', 'ex.WithMessage("*null result*")')

with open('src/Tests/GameClientCoverageTests.cs', 'w') as f:
    f.write(content)
print('Fixed')
