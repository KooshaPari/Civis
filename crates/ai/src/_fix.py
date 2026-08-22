p = "C:/Users/koosh/Civis/crates/ai/src/mcts.rs"
c = open(p).read()
old = (
    "root: "
    + chr(39)
    + "a mut Node, path: "
    + chr(38)
    + "[ActionId]) -> "
    + chr(39)
    + "a mut Node"
)
new = (
    "root: "
    + chr(38)
    + chr(39)
    + "a mut Node, path: "
    + chr(38)
    + "[ActionId]) -> "
    + chr(38)
    + chr(39)
    + "a mut Node"
)
c = c.replace(old, new)
open(p, "w").write(c)
print("patched")
