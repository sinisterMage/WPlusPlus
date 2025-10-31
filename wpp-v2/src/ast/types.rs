/// Type system for W++ dispatch and type aliases
use super::expr::Expr;

/// TypeDescriptor represents all types that can be used in function dispatch
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeDescriptor {
    /// Primitive types: i32, f64, ptr, bool, etc.
    Primitive(String),

    /// Entity (class) type by name
    Entity(String),

    /// Named object type (type alias)
    ObjectType(String),

    /// HTTP status code literal (e.g., 200, 404)
    HttpStatusLiteral(u16),

    /// HTTP status code range (e.g., 2xx means 200-299)
    HttpStatusRange(u16, u16),

    /// Wildcard pattern - matches anything
    Any,
}

impl TypeDescriptor {
    /// Calculate specificity for dispatch priority
    /// Higher specificity = more specific match
    pub fn specificity(&self) -> u32 {
        match self {
            TypeDescriptor::HttpStatusLiteral(_) => 100,
            TypeDescriptor::ObjectType(_) => 90,
            TypeDescriptor::Entity(_) => 90,
            TypeDescriptor::Primitive(p) if p != "ptr" => 80,
            TypeDescriptor::Primitive(_) => 70, // ptr is less specific
            TypeDescriptor::HttpStatusRange(_, _) => 50,
            TypeDescriptor::Any => 0,
        }
    }

    /// Check if this descriptor matches a given type
    pub fn matches(&self, other: &TypeDescriptor) -> bool {
        match (self, other) {
            // Exact matches
            (TypeDescriptor::Primitive(a), TypeDescriptor::Primitive(b)) => a == b,
            (TypeDescriptor::Entity(a), TypeDescriptor::Entity(b)) => a == b,
            (TypeDescriptor::ObjectType(a), TypeDescriptor::ObjectType(b)) => a == b,
            (TypeDescriptor::HttpStatusLiteral(a), TypeDescriptor::HttpStatusLiteral(b)) => a == b,

            // HTTP status code ranges
            (TypeDescriptor::HttpStatusRange(min, max), TypeDescriptor::HttpStatusLiteral(code)) => {
                code >= min && code <= max
            }
            (TypeDescriptor::HttpStatusLiteral(code), TypeDescriptor::HttpStatusRange(min, max)) => {
                code >= min && code <= max
            }

            // Wildcard matches everything
            (TypeDescriptor::Any, _) => true,
            (_, TypeDescriptor::Any) => true,

            _ => false,
        }
    }

    /// Convert to string representation for mangling
    pub fn to_mangle_string(&self) -> String {
        match self {
            TypeDescriptor::Primitive(p) => p.clone(),
            TypeDescriptor::Entity(e) => format!("entity_{}", e),
            TypeDescriptor::ObjectType(o) => format!("obj_{}", o),
            TypeDescriptor::HttpStatusLiteral(code) => format!("http_{}", code),
            TypeDescriptor::HttpStatusRange(min, max) => format!("http_{}xx", min / 100),
            TypeDescriptor::Any => "any".to_string(),
        }
    }
}

/// Object type definition (type alias for objects)
#[derive(Debug, Clone)]
pub struct ObjectTypeDefinition {
    pub name: String,
    pub fields: Vec<ObjectField>,
}

/// Field definition in an object type
#[derive(Debug, Clone)]
pub struct ObjectField {
    pub name: String,
    pub ty: FieldType,
}

/// Types that can be used in object fields
#[derive(Debug, Clone)]
pub enum FieldType {
    Int32,
    Int64,
    Float32,
    Float64,
    Bool,
    String,
    Array(Box<FieldType>),
    Object(String), // Nested object by type name
    Any,
}

impl FieldType {
    /// Parse a field type from string annotation
    pub fn from_string(s: &str) -> Self {
        match s {
            "i32" | "int" => FieldType::Int32,
            "i64" | "long" => FieldType::Int64,
            "f32" | "float" => FieldType::Float32,
            "f64" | "double" => FieldType::Float64,
            "bool" | "boolean" => FieldType::Bool,
            "string" | "str" => FieldType::String,
            _ => {
                // Check for array syntax: type[]
                if s.ends_with("[]") {
                    let inner = &s[..s.len() - 2];
                    FieldType::Array(Box::new(FieldType::from_string(inner)))
                } else {
                    // Assume it's a named object type
                    FieldType::Object(s.to_string())
                }
            }
        }
    }
}

/// Enhanced parameter with optional type pattern
#[derive(Debug, Clone)]
pub struct ParameterPattern {
    pub name: String,
    pub pattern: Option<TypePattern>,
}

/// Type pattern for parameter matching
#[derive(Debug, Clone)]
pub enum TypePattern {
    /// Type annotation: param: Type
    Type(TypeDescriptor),

    /// Value pattern: param: literal_value
    Value(Expr),
}

impl TypePattern {
    /// Get the type descriptor for this pattern
    pub fn to_type_descriptor(&self) -> Option<TypeDescriptor> {
        match self {
            TypePattern::Type(td) => Some(td.clone()),
            TypePattern::Value(expr) => {
                // Try to extract type from literal values
                match expr {
                    Expr::Literal(n) => {
                        // Check if it looks like HTTP status code
                        let num = *n as u16;
                        if num >= 100 && num < 600 {
                            Some(TypeDescriptor::HttpStatusLiteral(num))
                        } else {
                            Some(TypeDescriptor::Primitive("i32".to_string()))
                        }
                    }
                    _ => None,
                }
            }
        }
    }
}
