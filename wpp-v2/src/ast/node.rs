use super::expr::Expr;

#[derive(Debug, Clone)]
pub enum Node {
    Let {
        name: String,
        value: Expr,
        is_const: bool, // ✅ new field
        ty: Option<String>,
    },
    Expr(Expr),
    Entity(EntityNode),
}

#[derive(Debug, Clone)]
pub struct EntityNode {
    pub name: String,                    // entity name (e.g. "Dog")
    pub base: Option<String>,            // optional parent (for `alters`)
    pub members: Vec<EntityMember>,      // methods + fields
}

#[derive(Debug, Clone)]
pub enum EntityMember {
    Field {
        name: String,
        value: Expr,
    },
    Method {
        name: String,
        func: Expr, // Will be Expr::Funcy
    },
}