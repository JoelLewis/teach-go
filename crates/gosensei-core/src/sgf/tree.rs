use crate::types::{BoardSize, Color, Move, Point};

use super::parser::{parse_move, SgfParseError};

/// A node in an SGF game tree. Each node can have properties (move, setup
/// stones, comments) and zero or more child variations.
#[derive(Debug, Clone)]
pub struct SgfNode {
    /// Move played at this node, if any.
    pub mv: Option<(Color, Move)>,
    /// Comment text from C[] property.
    pub comment: Option<String>,
    /// Game name from GN[] property (typically only on root).
    pub game_name: Option<String>,
    /// "Good for Black" annotation (GB[]).
    pub good_for_black: bool,
    /// "Good for White" annotation (GW[]).
    pub good_for_white: bool,
    /// Child variations. A linear sequence has one child; branches have multiple.
    pub children: Vec<SgfNode>,
}

/// Root-level metadata for an SGF game tree, extracted from the root node.
#[derive(Debug, Clone)]
pub struct SgfTreeRoot {
    pub board_size: BoardSize,
    pub komi: f32,
    pub player_black: Option<String>,
    pub player_white: Option<String>,
    pub result: Option<String>,
    pub game_name: Option<String>,
    /// Setup stones from AB[] on the root node.
    pub setup_black: Vec<Point>,
    /// Setup stones from AW[] on the root node.
    pub setup_white: Vec<Point>,
    /// Player to move from PL[] on the root node.
    pub player_to_move: Option<Color>,
    /// The main variation tree (first child of root and descendants).
    pub root: SgfNode,
}

/// Parse an SGF string into a tree structure, preserving variations.
///
/// This handles the full SGF format including nested variations:
/// ```text
/// (;SZ[9]AB[dd](;B[ee](;W[ff])(;W[fg]))(;B[ef]))
/// ```
///
/// For SGF collections (multiple game trees in one string), use
/// [`parse_sgf_collection`].
pub fn parse_sgf_tree(input: &str) -> Result<SgfTreeRoot, SgfParseError> {
    let input = input.trim();
    if !input.starts_with('(') {
        return Err(SgfParseError::InvalidFormat(
            "SGF must start with '('".into(),
        ));
    }

    let mut pos = 1; // skip opening '('
    let bytes = input.as_bytes();

    // Parse the root node sequence and its variations
    let (root_node, root_props) = parse_node_sequence(bytes, &mut pos)?;

    Ok(SgfTreeRoot {
        board_size: root_props.board_size,
        komi: root_props.komi,
        player_black: root_props.player_black,
        player_white: root_props.player_white,
        result: root_props.result,
        game_name: root_node.game_name.clone(),
        setup_black: root_props.setup_black,
        setup_white: root_props.setup_white,
        player_to_move: root_props.player_to_move,
        root: root_node,
    })
}

/// Parse an SGF collection — multiple game trees in a single string.
///
/// SGF collections look like: `(;SZ[9];B[dd])(;SZ[9];B[ee])`
/// Each `(...)` is a separate game tree.
pub fn parse_sgf_collection(input: &str) -> Result<Vec<SgfTreeRoot>, SgfParseError> {
    let input = input.trim();
    let mut results = Vec::new();
    let mut start = 0;

    while start < input.len() {
        // Find the next game tree opening
        match input[start..].find('(') {
            Some(offset) => {
                let tree_start = start + offset;
                let tree_end = find_matching_paren(input.as_bytes(), tree_start)?;
                let tree_str = &input[tree_start..=tree_end];
                match parse_sgf_tree(tree_str) {
                    Ok(tree) => results.push(tree),
                    Err(e) => {
                        // Skip malformed trees in collections, but log via the error
                        if results.is_empty() {
                            return Err(e);
                        }
                        // Silently skip subsequent malformed trees
                    }
                }
                start = tree_end + 1;
            }
            None => break,
        }
    }

    if results.is_empty() {
        return Err(SgfParseError::InvalidFormat(
            "No valid game trees found".into(),
        ));
    }

    Ok(results)
}

/// Collected root-node properties during parsing.
struct RootProps {
    board_size: BoardSize,
    komi: f32,
    player_black: Option<String>,
    player_white: Option<String>,
    result: Option<String>,
    setup_black: Vec<Point>,
    setup_white: Vec<Point>,
    player_to_move: Option<Color>,
}

impl Default for RootProps {
    fn default() -> Self {
        Self {
            board_size: BoardSize::Nineteen,
            komi: 6.5,
            player_black: None,
            player_white: None,
            result: None,
            setup_black: Vec::new(),
            setup_white: Vec::new(),
            player_to_move: None,
        }
    }
}

/// Parse a sequence of nodes starting after `(` or `;`.
/// Returns the first node (with children attached) and root properties.
fn parse_node_sequence(
    bytes: &[u8],
    pos: &mut usize,
) -> Result<(SgfNode, RootProps), SgfParseError> {
    let mut root_props = RootProps::default();
    let mut nodes: Vec<SgfNode> = Vec::new();
    let mut is_root = true;

    while *pos < bytes.len() {
        skip_whitespace(bytes, pos);
        if *pos >= bytes.len() {
            break;
        }

        match bytes[*pos] {
            b';' => {
                *pos += 1;
                let node = parse_single_node(bytes, pos, if is_root { Some(&mut root_props) } else { None })?;
                nodes.push(node);
                is_root = false;
            }
            b'(' => {
                // Start of a variation branch
                *pos += 1; // skip '('
                let (child, _) = parse_node_sequence(bytes, pos)?;
                // Attach this variation to the last node in the sequence
                if let Some(last) = nodes.last_mut() {
                    last.children.push(child);
                }
            }
            b')' => {
                *pos += 1; // skip ')'
                break;
            }
            _ => {
                *pos += 1; // skip unexpected chars
            }
        }
    }

    // Chain nodes into a linked tree: first -> second -> ... -> last
    // Each node's children list becomes [next_in_sequence] plus any variation branches
    let first = chain_nodes(nodes);

    Ok((first, root_props))
}

/// Chain a flat list of sequential nodes into a tree.
/// Each node becomes the parent of the next, preserving existing children (variations).
fn chain_nodes(mut nodes: Vec<SgfNode>) -> SgfNode {
    if nodes.is_empty() {
        return SgfNode {
            mv: None,
            comment: None,
            game_name: None,
            good_for_black: false,
            good_for_white: false,
            children: Vec::new(),
        };
    }

    // Work backwards: last node has no next, each prior node gets next as a child
    let mut current = nodes.pop().unwrap();
    while let Some(mut parent) = nodes.pop() {
        // Insert the continuation as the first child (main line)
        parent.children.insert(0, current);
        current = parent;
    }
    current
}

/// Parse properties of a single node (between `;` and the next `;`, `(`, or `)`).
fn parse_single_node(
    bytes: &[u8],
    pos: &mut usize,
    mut root_props: Option<&mut RootProps>,
) -> Result<SgfNode, SgfParseError> {
    let mut node = SgfNode {
        mv: None,
        comment: None,
        game_name: None,
        good_for_black: false,
        good_for_white: false,
        children: Vec::new(),
    };

    while *pos < bytes.len() {
        skip_whitespace(bytes, pos);
        if *pos >= bytes.len() {
            break;
        }

        match bytes[*pos] {
            b';' | b'(' | b')' => break, // Next node or variation boundary
            c if c.is_ascii_uppercase() => {
                let prop_name = read_property_name(bytes, pos);
                let values = read_property_values(bytes, pos);

                match prop_name.as_str() {
                    "B" | "W" => {
                        let color = if prop_name == "B" {
                            Color::Black
                        } else {
                            Color::White
                        };
                        if let Some(val) = values.first() {
                            node.mv = Some((color, parse_move(val)));
                        }
                    }
                    "C" => {
                        node.comment = values.into_iter().next();
                    }
                    "GN" => {
                        node.game_name = values.into_iter().next();
                    }
                    "GB" => {
                        node.good_for_black = true;
                    }
                    "GW" => {
                        node.good_for_white = true;
                    }
                    // Root-only properties
                    "SZ" => {
                        if let Some(ref mut rp) = root_props
                            && let Some(val) = values.first()
                        {
                            let size: u8 = val.parse().unwrap_or(19);
                            rp.board_size = BoardSize::try_from(size)
                                .map_err(|_| SgfParseError::UnsupportedBoardSize(size))?;
                        }
                    }
                    "KM" => {
                        if let Some(ref mut rp) = root_props
                            && let Some(val) = values.first()
                        {
                            rp.komi = val.parse().unwrap_or(6.5);
                        }
                    }
                    "PB" => {
                        if let Some(ref mut rp) = root_props {
                            rp.player_black = values.into_iter().next();
                        }
                    }
                    "PW" => {
                        if let Some(ref mut rp) = root_props {
                            rp.player_white = values.into_iter().next();
                        }
                    }
                    "RE" => {
                        if let Some(ref mut rp) = root_props {
                            rp.result = values.into_iter().next();
                        }
                    }
                    "AB" => {
                        if let Some(ref mut rp) = root_props {
                            for val in &values {
                                if let Move::Play(p) = parse_move(val) {
                                    rp.setup_black.push(p);
                                }
                            }
                        }
                    }
                    "AW" => {
                        if let Some(ref mut rp) = root_props {
                            for val in &values {
                                if let Move::Play(p) = parse_move(val) {
                                    rp.setup_white.push(p);
                                }
                            }
                        }
                    }
                    "PL" => {
                        if let Some(ref mut rp) = root_props
                            && let Some(val) = values.first()
                        {
                            rp.player_to_move = match val.as_str() {
                                "B" => Some(Color::Black),
                                "W" => Some(Color::White),
                                _ => None,
                            };
                        }
                    }
                    _ => {} // Ignore unknown properties
                }
            }
            _ => {
                *pos += 1;
            }
        }
    }

    Ok(node)
}

fn read_property_name(bytes: &[u8], pos: &mut usize) -> String {
    let mut name = String::new();
    while *pos < bytes.len() && bytes[*pos].is_ascii_uppercase() {
        name.push(bytes[*pos] as char);
        *pos += 1;
    }
    name
}

fn read_property_values(bytes: &[u8], pos: &mut usize) -> Vec<String> {
    let mut values = Vec::new();
    while *pos < bytes.len() && bytes[*pos] == b'[' {
        *pos += 1; // skip '['
        let mut val = String::new();
        let mut escaped = false;
        while *pos < bytes.len() {
            let ch = bytes[*pos];
            *pos += 1;
            if escaped {
                val.push(ch as char);
                escaped = false;
            } else if ch == b'\\' {
                escaped = true;
            } else if ch == b']' {
                break;
            } else {
                val.push(ch as char);
            }
        }
        values.push(val);
    }
    values
}

fn skip_whitespace(bytes: &[u8], pos: &mut usize) {
    while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
}

/// Find the matching closing paren for an opening paren at `start`.
fn find_matching_paren(bytes: &[u8], start: usize) -> Result<usize, SgfParseError> {
    let mut depth = 0;
    let mut in_bracket = false;
    let mut escaped = false;
    for (i, &byte) in bytes.iter().enumerate().skip(start) {
        if escaped {
            escaped = false;
            continue;
        }
        match byte {
            b'\\' if in_bracket => escaped = true,
            b'[' if !in_bracket => in_bracket = true,
            b']' if in_bracket => in_bracket = false,
            b'(' if !in_bracket => depth += 1,
            b')' if !in_bracket => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i);
                }
            }
            _ => {}
        }
    }
    Err(SgfParseError::InvalidFormat(
        "Unmatched parenthesis".into(),
    ))
}

/// Helper: collect all moves from the main line (first child at each level).
impl SgfNode {
    pub fn main_line_moves(&self) -> Vec<(Color, Move)> {
        let mut moves = Vec::new();
        let mut current = self;
        loop {
            if let Some(mv) = current.mv {
                moves.push(mv);
            }
            if let Some(first_child) = current.children.first() {
                current = first_child;
            } else {
                break;
            }
        }
        moves
    }

    /// Count total variations (branches) in this subtree.
    pub fn variation_count(&self) -> usize {
        if self.children.len() <= 1 {
            self.children.first().map_or(0, |c| c.variation_count())
        } else {
            self.children.len()
                + self
                    .children
                    .iter()
                    .map(|c| c.variation_count())
                    .sum::<usize>()
        }
    }

    /// Get the depth (longest path from this node to a leaf).
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            0
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_linear_game() {
        let sgf = "(;SZ[9]KM[5.5];B[ee];W[cc])";
        let tree = parse_sgf_tree(sgf).unwrap();
        assert_eq!(tree.board_size, BoardSize::Nine);
        assert_eq!(tree.komi, 5.5);

        let moves = tree.root.main_line_moves();
        assert_eq!(moves.len(), 2);
        assert_eq!(moves[0].0, Color::Black);
        assert_eq!(moves[1].0, Color::White);
    }

    #[test]
    fn parse_with_variations() {
        // Root plays B[ee], then two variations for White
        let sgf = "(;SZ[9];B[ee](;W[cc])(;W[dd]))";
        let tree = parse_sgf_tree(sgf).unwrap();

        let moves = tree.root.main_line_moves();
        assert_eq!(moves.len(), 2); // B[ee] then W[cc] (main line = first variation)

        // The node with B[ee] should have two children (two White variations)
        let b_node = &tree.root; // root node (no move)
        let b_move = &b_node.children[0]; // ;B[ee]
        assert_eq!(b_move.children.len(), 2);
        assert_eq!(
            b_move.children[0].mv,
            Some((Color::White, Move::Play(Point::new(2, 2))))
        );
        assert_eq!(
            b_move.children[1].mv,
            Some((Color::White, Move::Play(Point::new(3, 3))))
        );
    }

    #[test]
    fn parse_nested_variations() {
        let sgf = "(;SZ[9];B[ee](;W[cc](;B[dd])(;B[ff]))(;W[gg]))";
        let tree = parse_sgf_tree(sgf).unwrap();

        // Main line: B[ee] -> W[cc] -> B[dd]
        let moves = tree.root.main_line_moves();
        assert_eq!(moves.len(), 3);

        // B[ee] node has 2 children (W[cc] and W[gg])
        let b_node = &tree.root.children[0];
        assert_eq!(b_node.children.len(), 2);

        // W[cc] node has 2 children (B[dd] and B[ff])
        let w_cc = &b_node.children[0];
        assert_eq!(w_cc.children.len(), 2);
    }

    #[test]
    fn parse_with_comments() {
        let sgf = "(;SZ[9]C[Root comment];B[ee]C[Good move])";
        let tree = parse_sgf_tree(sgf).unwrap();
        assert_eq!(tree.root.comment.as_deref(), Some("Root comment"));
        assert_eq!(
            tree.root.children[0].comment.as_deref(),
            Some("Good move")
        );
    }

    #[test]
    fn parse_with_annotations() {
        let sgf = "(;SZ[9];B[ee]GB[1]C[Correct!])";
        let tree = parse_sgf_tree(sgf).unwrap();
        let b_node = &tree.root.children[0];
        assert!(b_node.good_for_black);
        assert!(!b_node.good_for_white);
    }

    #[test]
    fn parse_setup_stones() {
        let sgf = "(;SZ[9]AB[dd][de]AW[ed][ee]PL[B])";
        let tree = parse_sgf_tree(sgf).unwrap();
        assert_eq!(tree.setup_black.len(), 2);
        assert_eq!(tree.setup_white.len(), 2);
        assert_eq!(tree.player_to_move, Some(Color::Black));
    }

    #[test]
    fn parse_collection() {
        let sgf = "(;SZ[9];B[ee])(;SZ[9];B[dd])";
        let trees = parse_sgf_collection(sgf).unwrap();
        assert_eq!(trees.len(), 2);
    }

    #[test]
    fn parse_game_name() {
        let sgf = "(;SZ[9]GN[Life and Death Problem 1];B[dd])";
        let tree = parse_sgf_tree(sgf).unwrap();
        assert_eq!(
            tree.game_name.as_deref(),
            Some("Life and Death Problem 1")
        );
    }

    #[test]
    fn variation_count() {
        let sgf = "(;SZ[9];B[ee](;W[cc])(;W[dd]))";
        let tree = parse_sgf_tree(sgf).unwrap();
        assert_eq!(tree.root.variation_count(), 2);
    }

    #[test]
    fn main_line_depth() {
        let sgf = "(;SZ[9];B[ee];W[cc];B[dd])";
        let tree = parse_sgf_tree(sgf).unwrap();
        assert_eq!(tree.root.depth(), 3); // 3 moves deep
    }

    #[test]
    fn parse_problem_sgf_with_variations() {
        // A typical tsumego problem SGF
        let sgf = "(;SZ[9]AB[aa][ba][ab]AW[bb][cb][bc]PL[B]\
                    (;B[ac]C[Correct!]GB[1]\
                      (;W[bd];B[cc]C[Black lives]))\
                    (;B[bd]C[Wrong - doesn't make two eyes]))";
        let tree = parse_sgf_tree(sgf).unwrap();
        assert_eq!(tree.setup_black.len(), 3);
        assert_eq!(tree.setup_white.len(), 3);
        assert_eq!(tree.player_to_move, Some(Color::Black));

        // Root should have 2 children: B[ac] (correct) and B[bd] (wrong)
        assert_eq!(tree.root.children.len(), 2);
        let correct = &tree.root.children[0];
        assert!(correct.good_for_black);
        assert_eq!(correct.comment.as_deref(), Some("Correct!"));

        let wrong = &tree.root.children[1];
        assert!(!wrong.good_for_black);
        assert!(wrong.comment.as_deref().unwrap().contains("Wrong"));
    }
}
