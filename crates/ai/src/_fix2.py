p = "C:/Users/koosh/Civis/crates/ai/src/mcts.rs"
c = open(p).read()
c = c.replace(
    "if leaf.is_terminal || leaf.visits == 0 { return None; }",
    "if leaf.is_terminal { return None; }",
)
c = c.replace(
    "pub fn search(&mut self, state: &G) {",
    "pub fn search(&mut self, state: &G) { if state.is_terminal() { return; }",
)
open(p, "w").write(c)
print("patched")
