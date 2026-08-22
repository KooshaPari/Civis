p = "C:/Users/koosh/Civis/crates/ai/src/mcts.rs"
c = open(p).read()
c = c.replace(
    "Self::backup(child, rest, 1.0 - reward);", "Self::backup(child, rest, reward);"
)
open(p, "w").write(c)
