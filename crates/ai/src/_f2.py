p = "C:/Users/koosh/Civis/crates/ai/src/mcts.rs"
c = open(p).read()
c = c.replace("node: &mut Node,", "mut node: &mut Node,")
open(p, "w").write(c)
