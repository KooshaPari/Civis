p = "C:/Users/koosh/Civis/crates/ai/src/mcts.rs"
c = open(p).read()
c = c.replace(
    "pub action: ActionId,",
    "/// The action that led to this node." + chr(10) + "    pub action: ActionId,",
)
c = c.replace(
    "pub total_value: f64,",
    "/// Total value accumulated from rollouts."
    + chr(10)
    + "    pub total_value: f64,",
)
c = c.replace(
    "pub visits: usize,",
    "/// Number of visits to this node." + chr(10) + "    pub visits: usize,",
)
c = c.replace(
    "pub children: "
    + chr(38)
    + "HashMapActionId, Node Child nodes indexed by action."
    + chr(10)
    + "    pub children: "
    + chr(38)
    + "HashMapActionId, Node is_terminal: bool,",
    "/// Whether this node represents a terminal state."
    + chr(10)
    + "    pub is_terminal: bool,",
)
c = c.replace(
    "    pub fn new(action: ActionId, is_terminal: bool) - {",
    "    /// Create a new leaf node."
    + chr(10)
    + "    pub fn new(action: ActionId, is_terminal: bool) - {",
)
c = c.replace(
    "    pub fn q_mean" + chr(38) + "self) - {",
    "    /// Mean action value."
    + chr(10)
    + "    pub fn q_mean"
    + chr(38)
    + "self) - {",
)
c = c.replace(
    "    pub fn ucb1" + chr(38) + "self, exploration: f64, parent_visits: usize) - {",
    "    /// Upper Confidence Bound for Trees (UCB1)."
    + chr(10)
    + "    pub fn ucb1"
    + chr(38)
    + "self, exploration: f64, parent_visits: usize) - {",
)
c = c.replace(
    "    pub fn new(state: " + chr(38) + "G, config: MctsConfig) - {",
    "    /// Create a new MCTS tree from a game state."
    + chr(10)
    + "    pub fn new(state: "
    + chr(38)
    + "G, config: MctsConfig) - {",
)
c = c.replace(
    "    pub fn root(" + chr(38) + "self) - { " + chr(38) + "self.root }",
    "    /// Return a reference to the root node."
    + chr(10)
    + "    pub fn root("
    + chr(38)
    + "self) - { "
    + chr(38)
    + "self.root }",
)
c = c.replace(
    "    pub fn search(" + chr(38) + "mut self, state: " + chr(38) + "G) {",
    "    /// Run the MCTS search iterations."
    + chr(10)
    + "    pub fn search("
    + chr(38)
    + "mut self, state: "
    + chr(38)
    + "G) {",
)
c = c.replace(
    "    pub fn best_action("
    + chr(38)
    + "self) -    /// Return the best action based on visit counts."
    + chr(10)
    + "    pub fn best_action("
    + chr(38)
    + "self) -    pub fn iterations("
    + chr(38)
    + "self) - { self.root.visits }",
    "    /// Total iterations executed."
    + chr(10)
    + "    pub fn iterations("
    + chr(38)
    + "self) - { self.root.visits }",
)
open(p, "w").write(c)
print("docs added")
