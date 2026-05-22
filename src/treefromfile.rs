// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Return the printable name of a Newick token type.
#[track_caller]
pub fn tree_ntt_str(ntt: NEWICKTOKENTYPE) -> &'static str {
    match ntt {
        NEWICKTOKENTYPE::NTT_Unknown => "Unknown",
        NEWICKTOKENTYPE::NTT_Lparen => "Lparen",
        NEWICKTOKENTYPE::NTT_Rparen => "Rparen",
        NEWICKTOKENTYPE::NTT_Colon => "Colon",
        NEWICKTOKENTYPE::NTT_Comma => "Comma",
        NEWICKTOKENTYPE::NTT_Semicolon => "Semicolon",
        NEWICKTOKENTYPE::NTT_String => "String",
        NEWICKTOKENTYPE::NTT_SingleQuotedString => "SingleQuotedString",
        NEWICKTOKENTYPE::NTT_DoubleQuotedString => "DoubleQuotedString",
        NEWICKTOKENTYPE::NTT_Comment => "Comment",
    }
}

/// Read the next Newick token from `file` (handles strings, quotes, comments, punctuation).
#[track_caller]
pub fn tree_get_token(file: &mut TextFile, bytes: uint) -> (NEWICKTOKENTYPE, String) {
    text_file_skip_white(file);

    let mut c = text_file_get_char_x(file);
    match c {
        b'(' => return (NEWICKTOKENTYPE::NTT_Lparen, "(".to_string()),
        b')' => return (NEWICKTOKENTYPE::NTT_Rparen, ")".to_string()),
        b':' => return (NEWICKTOKENTYPE::NTT_Colon, ":".to_string()),
        b';' => return (NEWICKTOKENTYPE::NTT_Semicolon, ";".to_string()),
        b',' => return (NEWICKTOKENTYPE::NTT_Comma, ",".to_string()),
        _ => {}
    }

    let tt = match c {
        b'\'' => {
            c = text_file_get_char_x(file);
            NEWICKTOKENTYPE::NTT_SingleQuotedString
        }
        b'"' => {
            c = text_file_get_char_x(file);
            NEWICKTOKENTYPE::NTT_DoubleQuotedString
        }
        b'[' => NEWICKTOKENTYPE::NTT_Comment,
        _ => NEWICKTOKENTYPE::NTT_String,
    };

    let mut token = Vec::new();
    loop {
        if tt != NEWICKTOKENTYPE::NTT_Comment {
            if token.len() < bytes as usize - 2 {
                token.push(c);
            } else {
                panic!(
                    "Tree::GetToken: input buffer too small, token so far='{}'",
                    String::from_utf8_lossy(&token)
                );
            }
        }

        let Some(next) = text_file_get_char(file) else {
            return (tt, String::from_utf8_lossy(&token).into_owned());
        };
        c = next;

        match tt {
            NEWICKTOKENTYPE::NTT_String => {
                if b"():;,".contains(&c) {
                    file.pushed_back = Some(c);
                    return (
                        NEWICKTOKENTYPE::NTT_String,
                        String::from_utf8_lossy(&token).into_owned(),
                    );
                }
                if (c as char).is_ascii_whitespace() {
                    return (
                        NEWICKTOKENTYPE::NTT_String,
                        String::from_utf8_lossy(&token).into_owned(),
                    );
                }
            }
            NEWICKTOKENTYPE::NTT_SingleQuotedString => {
                if c == b'\'' {
                    return (
                        NEWICKTOKENTYPE::NTT_String,
                        String::from_utf8_lossy(&token).into_owned(),
                    );
                }
            }
            NEWICKTOKENTYPE::NTT_DoubleQuotedString => {
                if c == b'"' {
                    return (
                        NEWICKTOKENTYPE::NTT_String,
                        String::from_utf8_lossy(&token).into_owned(),
                    );
                }
            }
            NEWICKTOKENTYPE::NTT_Comment => {
                if c == b']' {
                    return tree_get_token(file, bytes);
                }
            }
            _ => panic!("Tree::GetToken, invalid TT={:?}", tt),
        }
    }
}

/// Load a Newick tree from the given file path into `tree`.
#[track_caller]
pub fn tree_from_file_l143(tree: &mut Tree, file_name: &str) {
    let mut file = text_file_text_file_l5(file_name, false);
    tree_from_file_l150(tree, &mut file);
    text_file_destructor_text_file(&mut file);
}

/// Parse a Newick tree from `file` into `tree`, converting to unrooted if a third group is present.
#[track_caller]
pub fn tree_from_file_l150(tree: &mut Tree, file: &mut TextFile) {
    tree_create_rooted(tree);

    let root_edge_length = tree_get_group_from_file(tree, file, 0);
    let (ntt, token) = tree_get_token(file, 16);

    if ntt == NEWICKTOKENTYPE::NTT_Semicolon {
        tree_validate(tree);
        return;
    }

    if ntt != NEWICKTOKENTYPE::NTT_Comma {
        panic!("Tree::FromFile, expected ';' or ',', got '{token}'");
    }

    let third_node = tree_unroot_from_file(tree);
    let third_edge_length = tree_get_group_from_file(tree, file, third_node);
    if let Some(edge_length) = third_edge_length {
        tree_set_edge_length(tree, 0, third_node, edge_length);
    }
    let _ = root_edge_length;
    tree_validate(tree);
}

/// Parse one Newick group (leaf or `(left,right)`) into `node_index`; returns its edge length if present.
#[track_caller]
pub fn tree_get_group_from_file(
    tree: &mut Tree,
    file: &mut TextFile,
    node_index: uint,
) -> Option<f64> {
    let (mut ntt, mut token) = tree_get_token(file, 1024);

    if ntt == NEWICKTOKENTYPE::NTT_String {
        tree_set_leaf_name(tree, node_index, &token);
    } else if ntt == NEWICKTOKENTYPE::NTT_Lparen {
        let left = tree_append_branch(tree, node_index);
        let right = left + 1;

        if let Some(edge_length) = tree_get_group_from_file(tree, file, left) {
            tree_set_edge_length(tree, node_index, left, edge_length);
        }

        (ntt, token) = tree_get_token(file, 1024);
        if ntt != NEWICKTOKENTYPE::NTT_Comma {
            panic!("Tree::GetGroupFromFile, expected ',', got '{token}'");
        }

        if let Some(edge_length) = tree_get_group_from_file(tree, file, right) {
            tree_set_edge_length(tree, node_index, right, edge_length);
        }

        (ntt, token) = tree_get_token(file, 1024);
        if ntt == NEWICKTOKENTYPE::NTT_Rparen {
        } else if ntt == NEWICKTOKENTYPE::NTT_Comma {
            file.pushed_back = Some(b',');
            return None;
        } else {
            panic!("Tree::GetGroupFromFile, expected ')' or ',', got '{token}'");
        }
    } else {
        panic!("Tree::GetGroupFromFile, expected '(' or leaf name, got '{token}'");
    }

    if text_file_skip_white_x(file) {
        return None;
    }
    let c = text_file_get_char_x(file);
    if c == b':' {
        let (edge_ntt, edge_token) = tree_get_token(file, 1024);
        if edge_ntt != NEWICKTOKENTYPE::NTT_String {
            panic!("Tree::GetGroupFromFile, expected edge length, got '{edge_token}'");
        }
        return Some(edge_token.parse::<f64>().unwrap_or(0.0));
    }
    file.pushed_back = Some(c);
    None
}
