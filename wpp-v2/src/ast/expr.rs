use super::node::Node; // 👈 to use Node inside Expr
use super::types::{ParameterPattern, TypeDescriptor};

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(i32),
    BoolLiteral(bool),
    StringLiteral(String),
    Variable(String),
        TypedLiteral { value: String, ty: String },
    BinaryOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    If {
        cond: Box<Expr>,
        then_branch: Vec<Node>,
        else_branch: Option<Vec<Node>>,
    },
    While {
        cond: Box<Expr>,
        body: Vec<Node>,
    },
    For {
        init: Option<Box<Node>>,
        cond: Option<Box<Expr>>,
        post: Option<Box<Expr>>,
        body: Vec<Node>,
    },
    Break,
    Continue,
    Switch {
    expr: Box<Expr>,
    cases: Vec<(Expr, Vec<Node>)>, // Each case: condition + block
    default: Option<Vec<Node>>,    // Optional default block
},
    TryCatch {
        try_block: Vec<Node>,
        catch_var: Option<String>,
        catch_block: Vec<Node>,
        finally_block: Option<Vec<Node>>, // ✅ NEW
    },
    Throw {
        expr: Box<Expr>,
    },
    Funcy {
        name: String,
        params: Vec<String>, // Kept for backward compatibility - stores "name:type" format
        params_patterns: Option<Vec<ParameterPattern>>, // ✅ NEW: Enhanced pattern support
        body: Vec<Node>,
        is_async: bool, // ✅ Added this field
        return_type: Option<TypeDescriptor>, // ✅ NEW: Optional return type annotation
    },
    Return(Option<Box<Expr>>),
    Await(Box<Expr>), // ✅ new expression kind
    ArrayLiteral(Vec<Expr>),
    ObjectLiteral {
        fields: Vec<(String, Expr)>,
        type_name: Option<String>, // ✅ NEW: Track object type for dispatch
    },
    NewInstance {
        entity: String,
        args: Vec<Expr>,
    },
}

