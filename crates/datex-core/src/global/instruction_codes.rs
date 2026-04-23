use num_enum::TryFromPrimitive;
use strum::Display;
use strum_macros::EnumIter;

#[allow(non_camel_case_types)]
#[derive(
    EnumIter,
    Debug,
    Eq,
    PartialEq,
    TryFromPrimitive,
    Copy,
    Clone,
    Display,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum InstructionCode {
    // flow instructions 0x00 - 0x0f
    STATEMENTS,       // statements block
    SHORT_STATEMENTS, // optimized statements block with up to 255 instructions
    UNBOUNDED_STATEMENTS,
    UNBOUNDED_STATEMENTS_END, // end of statements block (only needed for unbounded blocks)

    APPLY_ZERO,
    APPLY_SINGLE,
    APPLY,

    GET_PROPERTY_DYNAMIC, // get property with arbitrary key value
    GET_PROPERTY_INDEX,   // get property with integer index
    GET_PROPERTY_TEXT,    // get property with text key

    TAKE_PROPERTY_DYNAMIC, // take property with arbitrary key value
    TAKE_PROPERTY_INDEX,   // take property with integer index
    TAKE_PROPERTY_TEXT,    // take property with text key

    SET_PROPERTY_DYNAMIC, // set property with arbitrary key value
    SET_PROPERTY_INDEX,   // set property with integer index
    SET_PROPERTY_TEXT,    // set property with text key

    MATCHES, // matches

    STRUCTURAL_EQUAL,     // ==
    NOT_STRUCTURAL_EQUAL, // !=
    EQUAL,                // ===
    NOT_EQUAL,            // !==
    IS,                   // is

    ADD,      // +
    SUBTRACT, // -
    MULTIPLY, // *
    DIVIDE,   // /
    MODULO,   // %
    POWER,    // ^

    AND,
    OR,
    NOT,

    UNARY_PLUS,
    UNARY_MINUS,
    BITWISE_NOT, // ~

    INCREMENT,
    DECREMENT,

    // pointers & variables 0xa0 - 0xbf

    // stack value
    // TODO #669: refactor with stack variable system?
    CLONE_STACK_VALUE, // clone #stack[i]   0x0000-0x00ff = variables passed on between scopes, 0x0100-0xfdff = normal variables, 0xfe00-0xffff = it variables (#it.0, #it.1, ...) for function arguments
    BORROW_STACK_VALUE, // &#stack[i]
    GET_STACK_VALUE_SHARED_REF, // '#stack[i]
    GET_STACK_VALUE_SHARED_REF_MUT, // 'mut #stack[i]
    SET_STACK_VALUE,   // #stack[i] = ...
    MODIFY_STACK_VALUE, // #stack[i] += ..., ...
    PUSH_TO_STACK,     // #stack += ...
    PUSH_TO_STACK_MULTIPLE, // #stack ...+= [x]
    TAKE_STACK_VALUE,  // #stack[i]

    GET_INTERNAL_SLOT, // e.g. #endpoint

    // Note: fix to sync with RawPointerAddress
    REQUEST_REMOTE_SHARED_REF,     // '$x
    REQUEST_REMOTE_SHARED_REF_MUT, // 'mut $x

    GET_INTERNAL_SHARED_REF, // '$y, containing globally unique internal id
    GET_LOCAL_SHARED_REF, // '$x, containing only the id, origin id is inferred from sender

    SHARED_REF,            // '/'mut $1234
    SHARED_REF_WITH_VALUE, // '/'mut $1234 mut [value]

    PERFORM_MOVE, // PERFORM_MOVE(3) $a, $b, $c (indicates two executing endpoint that pointers should be moved from sender to local, triggers MOVE)
    MOVE, // MOVE(3) $a->$a2, $b->$b2, $c->$c2 (indicates that pointers should be moved from local to receiver, triggered by PERFORM_MOVE, includes mapping to new ids)

    GET_SHARED_REF,     // dynamic 'x
    GET_SHARED_REF_MUT, // dynamic 'mut x

    CREATE_SHARED,     // shared x
    CREATE_SHARED_MUT, // shared mut x

    SET_SHARED_CONTAINER_VALUE, // *x = 10;

    UNBOX, // *x

    /// type byte codes --> switch to Type Space
    TYPED_VALUE,
    TYPE_EXPRESSION, // type()

    // ...

    // values 0xc0 - 0xdf
    TEXT,
    INT_8, // byte
    INT_16,
    INT_32,
    INT_64,
    INT_128,
    INT_BIG,
    INT, // default integer (unsized)

    RANGE,

    UINT_8, // u8
    UINT_16,
    UINT_32,
    UINT_64,
    UINT_128,

    DECIMAL_F32,
    DECIMAL_F64,
    DECIMAL_BIG,
    DECIMAL_AS_INT_32,
    DECIMAL_AS_INT_16,

    DECIMAL, // default decimal (unsized)

    TRUE,
    FALSE,
    NULL,

    SHORT_TEXT, // string with max. 255 characters

    ENDPOINT,

    // lists and maps 0xe0 - 0xef
    LIST,       // (1,2,3)
    SHORT_LIST, // (1,2,3) - optimized short list with up to 255 elements
    MAP,        // (a:1, b:2)
    SHORT_MAP,  // {a:1, b:2} - optimized short map with up to 255 elements

    KEY_VALUE_SHORT_TEXT,
    KEY_VALUE_DYNAMIC, // for object elements with dynamic key

    REMOTE_EXECUTION, // ::
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::info;
    use strum::IntoEnumIterator;

    #[ignore]
    #[test]
    fn test_instruction_code_values() {
        // print a list of all instruction codes and their values for debugging purposes
        for code in InstructionCode::iter() {
            info!("{:?} = {:2X}", code, code as u8);
        }
    }
}
