use gosensei_core::sgf::tree::{SgfNode, SgfTreeRoot, parse_sgf_collection, parse_sgf_tree};
use gosensei_core::types::{Color, Move};

use crate::problem::{Problem, ProblemCategory, ProblemSource, ResponseBranch, SolutionNode};

/// Result of importing problems from an SGF string.
#[derive(Debug)]
pub struct ImportResult {
    pub problems: Vec<Problem>,
    pub errors: Vec<String>,
}

/// Import problems from an SGF string that may contain one or more game trees.
///
/// Each game tree is treated as a separate problem. The first move determines
/// the player color. Variations become the solution tree.
///
/// `default_difficulty`: fallback difficulty when no `DL[]` is present.
/// Pass `None` to use the hardcoded default (22.0).
pub fn import_from_sgf(sgf_text: &str, default_difficulty: Option<f64>) -> ImportResult {
    let mut result = ImportResult {
        problems: Vec::new(),
        errors: Vec::new(),
    };

    // Try as collection first, then as single tree
    let trees = match parse_sgf_collection(sgf_text) {
        Ok(trees) => trees,
        Err(_) => match parse_sgf_tree(sgf_text) {
            Ok(tree) => vec![tree],
            Err(e) => {
                result.errors.push(format!("Failed to parse SGF: {e}"));
                return result;
            }
        },
    };

    for (i, tree) in trees.into_iter().enumerate() {
        match convert_tree_to_problem(tree, i, default_difficulty) {
            Ok(problem) => result.problems.push(problem),
            Err(e) => result.errors.push(format!("Tree {}: {e}", i + 1)),
        }
    }

    result
}

/// Convert a parsed SGF tree into a Problem.
fn convert_tree_to_problem(
    tree: SgfTreeRoot,
    index: usize,
    default_difficulty: Option<f64>,
) -> Result<Problem, String> {
    let board_size = tree.board_size;

    // Determine player color from first move or PL[] property
    let player_color = determine_player_color(&tree);

    // Build the setup SGF from root's setup stones
    let setup_sgf = crate::problem::points_to_setup_sgf(
        board_size.size(),
        &tree.setup_black,
        &tree.setup_white,
    );

    // Convert the tree variations into SolutionNode trees
    let solutions = build_solution_tree(&tree.root, player_color);

    if solutions.is_empty() {
        return Err("No solution moves found".into());
    }

    // Infer category from metadata
    let category = infer_category(&tree);

    // Build prompt from comments and metadata
    let prompt = build_prompt(&tree, player_color);

    // Use DL[] if present (0–5 scale → 1.0–25.0 rank), else caller's default
    let difficulty = if let Some(dl) = tree.root.difficulty_level {
        // DL[0] = hardest (1.0), DL[5] = easiest (25.0)
        let clamped = (dl as f64).clamp(0.0, 5.0);
        1.0 + (clamped / 5.0) * 24.0
    } else {
        default_difficulty.unwrap_or(22.0)
    };

    // Extract tags from game name and comments
    let tags = build_tags(&tree, index);

    Ok(Problem {
        id: 0, // Will be assigned by DB
        setup_sgf,
        board_size,
        player_color,
        solutions,
        category,
        difficulty,
        source: ProblemSource::Imported,
        source_game_id: None,
        prompt,
        tags,
    })
}

/// Determine the player color from PL[] or first move.
fn determine_player_color(tree: &SgfTreeRoot) -> Color {
    if let Some(color) = tree.player_to_move {
        return color;
    }

    // Walk the tree to find the first move
    fn find_first_move(node: &SgfNode) -> Option<Color> {
        if let Some((color, _)) = node.mv {
            return Some(color);
        }
        for child in &node.children {
            if let Some(color) = find_first_move(child) {
                return Some(color);
            }
        }
        None
    }

    find_first_move(&tree.root).unwrap_or(Color::Black)
}

/// Recursively build SolutionNode trees from SGF variation tree.
///
/// Nodes where the move color matches `player_color` become SolutionNodes.
/// Nodes where the move color is the opponent become ResponseBranches.
fn build_solution_tree(node: &SgfNode, player_color: Color) -> Vec<SolutionNode> {
    let mut solutions = Vec::new();

    for child in &node.children {
        if let Some((color, Move::Play(point))) = child.mv
            && color == player_color
            && (is_correct_move(child) || node.children.len() == 1)
        {
            let responses = build_response_branches(child, player_color);
            solutions.push(SolutionNode { point, responses });
        }
    }

    // If no explicitly correct moves were found, treat all player moves as correct
    // (common in problem files that use main-line-is-correct convention)
    if solutions.is_empty() {
        for child in &node.children {
            if let Some((color, Move::Play(point))) = child.mv
                && color == player_color
            {
                let responses = build_response_branches(child, player_color);
                solutions.push(SolutionNode { point, responses });
            }
        }
    }

    solutions
}

/// Build response branches from an opponent's possible replies.
fn build_response_branches(player_node: &SgfNode, player_color: Color) -> Vec<ResponseBranch> {
    let opponent_color = player_color.opponent();
    let mut branches = Vec::new();

    for child in &player_node.children {
        if let Some((color, Move::Play(point))) = child.mv
            && color == opponent_color
        {
            let correct_moves = build_solution_tree(child, player_color);
            branches.push(ResponseBranch {
                opponent_move: point,
                correct_moves,
            });
        }
    }

    branches
}

/// Check if a node is marked as a correct answer.
fn is_correct_move(node: &SgfNode) -> bool {
    // GB[] or GW[] annotations
    if node.good_for_black || node.good_for_white {
        return true;
    }

    // Comment-based markers
    if let Some(ref comment) = node.comment {
        let lower = comment.to_lowercase();
        if lower.contains("correct")
            || lower.contains("right")
            || lower.contains("good")
            || lower.contains("best")
        {
            return true;
        }
    }

    false
}

/// Infer the problem category from SGF metadata.
pub fn infer_category(tree: &SgfTreeRoot) -> ProblemCategory {
    let mut text = String::new();

    if let Some(ref name) = tree.game_name {
        text.push_str(&name.to_lowercase());
        text.push(' ');
    }

    // Check root node comment
    if let Some(ref comment) = tree.root.comment {
        text.push_str(&comment.to_lowercase());
    }

    infer_category_from_text(&text)
}

/// Map text keywords to a ProblemCategory.
fn infer_category_from_text(text: &str) -> ProblemCategory {
    // Check in priority order — life/death is most specific
    if text.contains("life")
        || text.contains("death")
        || text.contains("kill")
        || text.contains("live")
        || text.contains("dead")
        || text.contains("eye")
        || text.contains("tsumego")
        || text.contains("semeai")
    {
        return ProblemCategory::LifeDeath;
    }

    if text.contains("tesuji") || text.contains("trick") || text.contains("clever") {
        return ProblemCategory::Tesuji;
    }

    if text.contains("ko") && !text.contains("komi") {
        return ProblemCategory::Ko;
    }

    if text.contains("endgame") || text.contains("yose") {
        return ProblemCategory::Endgame;
    }

    if text.contains("opening") || text.contains("fuseki") {
        return ProblemCategory::Opening;
    }

    if text.contains("shape") || text.contains("good form") {
        return ProblemCategory::Shape;
    }

    if text.contains("capture") || text.contains("capturing race") {
        return ProblemCategory::CapturingRace;
    }

    if text.contains("direction") || text.contains("strategy") {
        return ProblemCategory::Direction;
    }

    // Default to LifeDeath — most common problem type
    ProblemCategory::LifeDeath
}

/// Build a human-readable prompt from the tree metadata.
fn build_prompt(tree: &SgfTreeRoot, player_color: Color) -> String {
    // Use root comment if it looks like a prompt
    if let Some(ref comment) = tree.root.comment {
        let trimmed = comment.trim();
        if !trimmed.is_empty() && trimmed.len() < 200 {
            return trimmed.to_string();
        }
    }

    // Use game name as prompt if available
    if let Some(ref name) = tree.game_name {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    // Default prompt
    let color_name = match player_color {
        Color::Black => "Black",
        Color::White => "White",
    };
    format!("{color_name} to play")
}

/// Extract tags from metadata.
fn build_tags(tree: &SgfTreeRoot, index: usize) -> Vec<String> {
    let mut tags = vec!["imported".to_string()];

    if let Some(ref name) = tree.game_name {
        // Add sanitized game name as a tag if it's short
        let cleaned: String = name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-')
            .collect();
        let cleaned = cleaned.trim().to_lowercase();
        if !cleaned.is_empty() && cleaned.len() <= 40 {
            tags.push(cleaned);
        }
    }

    // Add problem number tag if in a collection
    if index > 0 {
        tags.push(format!("problem-{}", index + 1));
    }

    tags
}

#[cfg(test)]
mod tests {
    use super::*;
    use gosensei_core::types::{BoardSize, Point};

    #[test]
    fn import_simple_problem() {
        let sgf = "(;SZ[9]AB[dd][de]AW[ed][ee]PL[B];B[df]C[Correct!]GB[1])";
        let result = import_from_sgf(sgf, None);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert_eq!(result.problems.len(), 1);

        let p = &result.problems[0];
        assert_eq!(p.board_size, BoardSize::Nine);
        assert_eq!(p.player_color, Color::Black);
        assert_eq!(p.solutions.len(), 1);
        assert_eq!(p.solutions[0].point, Point::new(5, 3)); // df = col d(3), row f(5)
        assert_eq!(p.source, ProblemSource::Imported);
    }

    #[test]
    fn import_with_variations() {
        let sgf = "(;SZ[9]AB[aa][ba]AW[bb][cb]PL[B]\
                    (;B[ac]GB[1](;W[bd];B[cc]))\
                    (;B[bd]C[Wrong]))";
        let result = import_from_sgf(sgf, None);
        assert!(result.errors.is_empty());
        assert_eq!(result.problems.len(), 1);

        let p = &result.problems[0];
        // Only B[ac] should be a solution (marked GB[1])
        assert_eq!(p.solutions.len(), 1);
        assert_eq!(p.solutions[0].point, Point::new(2, 0)); // ac

        // Check response branch exists (W[bd])
        assert_eq!(p.solutions[0].responses.len(), 1);
        // Check follow-up (B[cc])
        assert_eq!(p.solutions[0].responses[0].correct_moves.len(), 1);
    }

    #[test]
    fn import_collection() {
        let sgf = "(;SZ[9]PL[B];B[ee])(;SZ[9]PL[B];B[dd])";
        let result = import_from_sgf(sgf, None);
        assert_eq!(result.problems.len(), 2);
    }

    #[test]
    fn infer_category_life_death() {
        assert_eq!(
            infer_category_from_text("life and death problem 1"),
            ProblemCategory::LifeDeath
        );
        assert_eq!(
            infer_category_from_text("can black kill?"),
            ProblemCategory::LifeDeath
        );
        assert_eq!(
            infer_category_from_text("tsumego collection"),
            ProblemCategory::LifeDeath
        );
    }

    #[test]
    fn infer_category_tesuji() {
        assert_eq!(
            infer_category_from_text("tesuji problem set"),
            ProblemCategory::Tesuji
        );
    }

    #[test]
    fn infer_category_ko() {
        assert_eq!(
            infer_category_from_text("ko fight problem"),
            ProblemCategory::Ko
        );
        // "komi" should not trigger Ko
        assert_ne!(infer_category_from_text("komi rules"), ProblemCategory::Ko);
    }

    #[test]
    fn infer_category_default() {
        assert_eq!(infer_category_from_text(""), ProblemCategory::LifeDeath);
    }

    #[test]
    fn build_prompt_from_comment() {
        let tree = SgfTreeRoot {
            board_size: BoardSize::Nine,
            komi: 6.5,
            player_black: None,
            player_white: None,
            result: None,
            game_name: None,
            setup_black: vec![],
            setup_white: vec![],
            player_to_move: Some(Color::Black),
            root: SgfNode {
                mv: None,
                comment: Some("Black to play and live".into()),
                game_name: None,
                good_for_black: false,
                good_for_white: false,
                difficulty_level: None,
                children: vec![],
            },
        };
        assert_eq!(build_prompt(&tree, Color::Black), "Black to play and live");
    }

    #[test]
    fn import_no_solution_returns_error() {
        let sgf = "(;SZ[9])"; // No moves at all
        let result = import_from_sgf(sgf, None);
        assert_eq!(result.problems.len(), 0);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn import_with_game_name_category() {
        let sgf = "(;SZ[9]GN[Tesuji Problem 5]PL[B];B[ee])";
        let result = import_from_sgf(sgf, None);
        assert_eq!(result.problems.len(), 1);
        assert_eq!(result.problems[0].category, ProblemCategory::Tesuji);
    }
}
