//TODO: find a complete list
pub const RESERVED: &[&str] = &[
    // control flow
    "break",
    "if",
    "else",
    "continue",
    "goto",
    "do",
    "while",
    "for",
    "switch",
    "case",
    // types and values
    "void",
    "unsigned",
    "signed",
    "bool",
    "char",
    "int",
    "uint",
    "long",
    "float",
    "double",
    "char8_t",
    "wchar_t",
    "true",
    "false",
    "nullptr",
    "union",
    "class",
    "struct",
    "enum",
    // other
    "main",
    "using",
    "decltype",
    "sizeof",
    "typeof",
    "typedef",
    "explicit",
    "export",
    "friend",
    "namespace",
    "operator",
    "public",
    "template",
    "typename",
    "typeid",
    "co_await",
    "co_return",
    "co_yield",
    "module",
    "import",
    "ray_data",
    "vec_step",
    "visible",
    "as_type",
    "this",
    // qualifiers
    "mutable",
    "static",
    "volatile",
    "restrict",
    "const",
    "non-temporal",
    "dereferenceable",
    "invariant",
    // exceptions
    "throw",
    "try",
    "catch",
    // operators
    "const_cast",
    "dynamic_cast",
    "reinterpret_cast",
    "static_cast",
    "new",
    "delete",
    "and",
    "and_eq",
    "bitand",
    "bitor",
    "compl",
    "not",
    "not_eq",
    "or",
    "or_eq",
    "xor",
    "xor_eq",
    "compl",
    // Metal-specific
    "constant",
    "device",
    "threadgroup",
    "threadgroup_imageblock",
    "kernel",
    "compute",
    "vertex",
    "fragment",
    "read_only",
    "write_only",
    "read_write",
    "auto",
    // Metal reserved types
    "llong",
    "ullong",
    "quad",
    "complex",
    "imaginary",
    // Metal constants
    "CHAR_BIT",
    "SCHAR_MAX",
    "SCHAR_MIN",
    "UCHAR_MAX",
    "CHAR_MAX",
    "CHAR_MIN",
    "USHRT_MAX",
    "SHRT_MAX",
    "SHRT_MIN",
    "UINT_MAX",
    "INT_MAX",
    "INT_MIN",
    "ULONG_MAX",
    "LONG_MAX",
    "LONG_MIN",
    "ULLONG_MAX",
    "LLONG_MAX",
    "LLONG_MIN",
    "FLT_DIG",
    "FLT_MANT_DIG",
    "FLT_MAX_10_EXP",
    "FLT_MAX_EXP",
    "FLT_MIN_10_EXP",
    "FLT_MIN_EXP",
    "FLT_RADIX",
    "FLT_MAX",
    "FLT_MIN",
    "FLT_EPSILON",
    "FLT_DECIMAL_DIG",
    "FP_ILOGB0",
    "FP_ILOGB0",
    "FP_ILOGBNAN",
    "FP_ILOGBNAN",
    "MAXFLOAT",
    "HUGE_VALF",
    "INFINITY",
    "NAN",
    "M_E_F",
    "M_LOG2E_F",
    "M_LOG10E_F",
    "M_LN2_F",
    "M_LN10_F",
    "M_PI_F",
    "M_PI_2_F",
    "M_PI_4_F",
    "M_1_PI_F",
    "M_2_PI_F",
    "M_2_SQRTPI_F",
    "M_SQRT2_F",
    "M_SQRT1_2_F",
    "HALF_DIG",
    "HALF_MANT_DIG",
    "HALF_MAX_10_EXP",
    "HALF_MAX_EXP",
    "HALF_MIN_10_EXP",
    "HALF_MIN_EXP",
    "HALF_RADIX",
    "HALF_MAX",
    "HALF_MIN",
    "HALF_EPSILON",
    "HALF_DECIMAL_DIG",
    "MAXHALF",
    "HUGE_VALH",
    "M_E_H",
    "M_LOG2E_H",
    "M_LOG10E_H",
    "M_LN2_H",
    "M_LN10_H",
    "M_PI_H",
    "M_PI_2_H",
    "M_PI_4_H",
    "M_1_PI_H",
    "M_2_PI_H",
    "M_2_SQRTPI_H",
    "M_SQRT2_H",
    "M_SQRT1_2_H",
    "DBL_DIG",
    "DBL_MANT_DIG",
    "DBL_MAX_10_EXP",
    "DBL_MAX_EXP",
    "DBL_MIN_10_EXP",
    "DBL_MIN_EXP",
    "DBL_RADIX",
    "DBL_MAX",
    "DBL_MIN",
    "DBL_EPSILON",
    "DBL_DECIMAL_DIG",
    "MAXDOUBLE",
    "HUGE_VAL",
    "M_E",
    "M_LOG2E",
    "M_LOG10E",
    "M_LN2",
    "M_LN10",
    "M_PI",
    "M_PI_2",
    "M_PI_4",
    "M_1_PI",
    "M_2_PI",
    "M_2_SQRTPI",
    "M_SQRT2",
    "M_SQRT1_2",
    // Naga utilities
    "DefaultConstructible",
    "clamped_lod_e",
];
