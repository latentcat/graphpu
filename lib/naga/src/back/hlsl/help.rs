/*!
Helpers for the hlsl backend

Important note about `Expression::ImageQuery`/`Expression::ArrayLength` and hlsl backend:

Due to implementation of `GetDimensions` function in hlsl (<https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/dx-graphics-hlsl-to-getdimensions>)
backend can't work with it as an expression.
Instead, it generates a unique wrapped function per `Expression::ImageQuery`, based on texture info and query function.
See `WrappedImageQuery` struct that represents a unique function and will be generated before writing all statements and expressions.
This allowed to works with `Expression::ImageQuery` as expression and write wrapped function.

For example:
```wgsl
let dim_1d = textureDimensions(image_1d);
```

```hlsl
int NagaDimensions1D(Texture1D<float4>)
{
   uint4 ret;
   image_1d.GetDimensions(ret.x);
   return ret.x;
}

int dim_1d = NagaDimensions1D(image_1d);
```
*/

use super::{super::FunctionCtx, BackendResult};
use crate::{arena::Handle, proc::NameKey};
use std::fmt::Write;

#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct WrappedArrayLength {
    pub(super) writable: bool,
}

#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct WrappedImageQuery {
    pub(super) dim: crate::ImageDimension,
    pub(super) arrayed: bool,
    pub(super) class: crate::ImageClass,
    pub(super) query: ImageQuery,
}

#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct WrappedConstructor {
    pub(super) ty: Handle<crate::Type>,
}

#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct WrappedStructMatrixAccess {
    pub(super) ty: Handle<crate::Type>,
    pub(super) index: u32,
}

#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct WrappedMatCx2 {
    pub(super) columns: crate::VectorSize,
}

/// HLSL backend requires its own `ImageQuery` enum.
///
/// It is used inside `WrappedImageQuery` and should be unique per ImageQuery function.
/// IR version can't be unique per function, because it's store mipmap level as an expression.
///
/// For example:
/// ```wgsl
/// let dim_cube_array_lod = textureDimensions(image_cube_array, 1);
/// let dim_cube_array_lod2 = textureDimensions(image_cube_array, 1);
/// ```
///
/// ```ir
/// ImageQuery {
///  image: [1],
///  query: Size {
///      level: Some(
///          [1],
///      ),
///  },
/// },
/// ImageQuery {
///  image: [1],
///  query: Size {
///      level: Some(
///          [2],
///      ),
///  },
/// },
/// ```
///
/// HLSL should generate only 1 function for this case.
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ImageQuery {
    Size,
    SizeLevel,
    NumLevels,
    NumLayers,
    NumSamples,
}

impl From<crate::ImageQuery> for ImageQuery {
    fn from(q: crate::ImageQuery) -> Self {
        use crate::ImageQuery as Iq;
        match q {
            Iq::Size { level: Some(_) } => ImageQuery::SizeLevel,
            Iq::Size { level: None } => ImageQuery::Size,
            Iq::NumLevels => ImageQuery::NumLevels,
            Iq::NumLayers => ImageQuery::NumLayers,
            Iq::NumSamples => ImageQuery::NumSamples,
        }
    }
}

impl<'a, W: Write> super::Writer<'a, W> {
    pub(super) fn write_image_type(
        &mut self,
        dim: crate::ImageDimension,
        arrayed: bool,
        class: crate::ImageClass,
    ) -> BackendResult {
        let access_str = match class {
            crate::ImageClass::Storage { .. } => "RW",
            _ => "",
        };
        let dim_str = dim.to_hlsl_str();
        let arrayed_str = if arrayed { "Array" } else { "" };
        write!(self.out, "{}Texture{}{}", access_str, dim_str, arrayed_str)?;
        match class {
            crate::ImageClass::Depth { multi } => {
                let multi_str = if multi { "MS" } else { "" };
                write!(self.out, "{}<float>", multi_str)?
            }
            crate::ImageClass::Sampled { kind, multi } => {
                let multi_str = if multi { "MS" } else { "" };
                let scalar_kind_str = kind.to_hlsl_str(4)?;
                write!(self.out, "{}<{}4>", multi_str, scalar_kind_str)?
            }
            crate::ImageClass::Storage { format, .. } => {
                let storage_format_str = format.to_hlsl_str();
                write!(self.out, "<{}>", storage_format_str)?
            }
        }
        Ok(())
    }

    pub(super) fn write_wrapped_array_length_function_name(
        &mut self,
        query: WrappedArrayLength,
    ) -> BackendResult {
        let access_str = if query.writable { "RW" } else { "" };
        write!(self.out, "NagaBufferLength{}", access_str,)?;

        Ok(())
    }

    /// Helper function that write wrapped function for `Expression::ArrayLength`
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/sm5-object-rwbyteaddressbuffer-getdimensions>
    pub(super) fn write_wrapped_array_length_function(
        &mut self,
        module: &crate::Module,
        wal: WrappedArrayLength,
        expr_handle: Handle<crate::Expression>,
        func_ctx: &FunctionCtx,
    ) -> BackendResult {
        use crate::back::INDENT;

        const ARGUMENT_VARIABLE_NAME: &str = "buffer";
        const RETURN_VARIABLE_NAME: &str = "ret";

        // Write function return type and name
        let ret_ty = func_ctx.info[expr_handle].ty.inner_with(&module.types);
        self.write_value_type(module, ret_ty)?;
        write!(self.out, " ")?;
        self.write_wrapped_array_length_function_name(wal)?;

        // Write function parameters
        write!(self.out, "(")?;
        let access_str = if wal.writable { "RW" } else { "" };
        writeln!(
            self.out,
            "{}ByteAddressBuffer {})",
            access_str, ARGUMENT_VARIABLE_NAME
        )?;
        // Write function body
        writeln!(self.out, "{{")?;

        // Write `GetDimensions` function.
        writeln!(self.out, "{}uint {};", INDENT, RETURN_VARIABLE_NAME)?;
        writeln!(
            self.out,
            "{}{}.GetDimensions({});",
            INDENT, ARGUMENT_VARIABLE_NAME, RETURN_VARIABLE_NAME
        )?;

        // Write return value
        writeln!(self.out, "{}return {};", INDENT, RETURN_VARIABLE_NAME)?;

        // End of function body
        writeln!(self.out, "}}")?;
        // Write extra new line
        writeln!(self.out)?;

        Ok(())
    }

    pub(super) fn write_wrapped_image_query_function_name(
        &mut self,
        query: WrappedImageQuery,
    ) -> BackendResult {
        let dim_str = query.dim.to_hlsl_str();
        let class_str = match query.class {
            crate::ImageClass::Sampled { multi: true, .. } => "MS",
            crate::ImageClass::Depth { multi: true } => "DepthMS",
            crate::ImageClass::Depth { multi: false } => "Depth",
            crate::ImageClass::Sampled { multi: false, .. } => "",
            crate::ImageClass::Storage { .. } => "RW",
        };
        let arrayed_str = if query.arrayed { "Array" } else { "" };
        let query_str = match query.query {
            ImageQuery::Size => "Dimensions",
            ImageQuery::SizeLevel => "MipDimensions",
            ImageQuery::NumLevels => "NumLevels",
            ImageQuery::NumLayers => "NumLayers",
            ImageQuery::NumSamples => "NumSamples",
        };

        write!(
            self.out,
            "Naga{}{}{}{}",
            class_str, query_str, dim_str, arrayed_str
        )?;

        Ok(())
    }

    /// Helper function that write wrapped function for `Expression::ImageQuery`
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/dx-graphics-hlsl-to-getdimensions>
    pub(super) fn write_wrapped_image_query_function(
        &mut self,
        module: &crate::Module,
        wiq: WrappedImageQuery,
        expr_handle: Handle<crate::Expression>,
        func_ctx: &FunctionCtx,
    ) -> BackendResult {
        use crate::{
            back::{COMPONENTS, INDENT},
            ImageDimension as IDim,
        };

        const ARGUMENT_VARIABLE_NAME: &str = "tex";
        const RETURN_VARIABLE_NAME: &str = "ret";
        const MIP_LEVEL_PARAM: &str = "mip_level";

        // Write function return type and name
        let ret_ty = func_ctx.info[expr_handle].ty.inner_with(&module.types);
        self.write_value_type(module, ret_ty)?;
        write!(self.out, " ")?;
        self.write_wrapped_image_query_function_name(wiq)?;

        // Write function parameters
        write!(self.out, "(")?;
        // Texture always first parameter
        self.write_image_type(wiq.dim, wiq.arrayed, wiq.class)?;
        write!(self.out, " {}", ARGUMENT_VARIABLE_NAME)?;
        // Mipmap is a second parameter if exists
        if let ImageQuery::SizeLevel = wiq.query {
            write!(self.out, ", uint {}", MIP_LEVEL_PARAM)?;
        }
        writeln!(self.out, ")")?;

        // Write function body
        writeln!(self.out, "{{")?;

        let array_coords = if wiq.arrayed { 1 } else { 0 };
        // extra parameter is the mip level count or the sample count
        let extra_coords = match wiq.class {
            crate::ImageClass::Storage { .. } => 0,
            crate::ImageClass::Sampled { .. } | crate::ImageClass::Depth { .. } => 1,
        };

        // GetDimensions Overloaded Methods
        // https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/dx-graphics-hlsl-to-getdimensions#overloaded-methods
        let (ret_swizzle, number_of_params) = match wiq.query {
            ImageQuery::Size | ImageQuery::SizeLevel => {
                let ret = match wiq.dim {
                    IDim::D1 => "x",
                    IDim::D2 => "xy",
                    IDim::D3 => "xyz",
                    IDim::Cube => "xy",
                };
                (ret, ret.len() + array_coords + extra_coords)
            }
            ImageQuery::NumLevels | ImageQuery::NumSamples | ImageQuery::NumLayers => {
                if wiq.arrayed || wiq.dim == IDim::D3 {
                    ("w", 4)
                } else {
                    ("z", 3)
                }
            }
        };

        // Write `GetDimensions` function.
        writeln!(self.out, "{}uint4 {};", INDENT, RETURN_VARIABLE_NAME)?;
        write!(
            self.out,
            "{}{}.GetDimensions(",
            INDENT, ARGUMENT_VARIABLE_NAME
        )?;
        match wiq.query {
            ImageQuery::SizeLevel => {
                write!(self.out, "{}, ", MIP_LEVEL_PARAM)?;
            }
            _ => match wiq.class {
                crate::ImageClass::Sampled { multi: true, .. }
                | crate::ImageClass::Depth { multi: true }
                | crate::ImageClass::Storage { .. } => {}
                _ => {
                    // Write zero mipmap level for supported types
                    write!(self.out, "0, ")?;
                }
            },
        }

        for component in COMPONENTS[..number_of_params - 1].iter() {
            write!(self.out, "{}.{}, ", RETURN_VARIABLE_NAME, component)?;
        }

        // write last parameter without comma and space for last parameter
        write!(
            self.out,
            "{}.{}",
            RETURN_VARIABLE_NAME,
            COMPONENTS[number_of_params - 1]
        )?;

        writeln!(self.out, ");")?;

        // Write return value
        writeln!(
            self.out,
            "{}return {}.{};",
            INDENT, RETURN_VARIABLE_NAME, ret_swizzle
        )?;

        // End of function body
        writeln!(self.out, "}}")?;
        // Write extra new line
        writeln!(self.out)?;

        Ok(())
    }

    pub(super) fn write_wrapped_constructor_function_name(
        &mut self,
        module: &crate::Module,
        constructor: WrappedConstructor,
    ) -> BackendResult {
        let name = module.types[constructor.ty].inner.hlsl_type_id(
            constructor.ty,
            &module.types,
            &module.constants,
            &self.names,
        )?;
        write!(self.out, "Construct{}", name)?;
        Ok(())
    }

    /// Helper function that write wrapped function for `Expression::Compose` for structures.
    pub(super) fn write_wrapped_constructor_function(
        &mut self,
        module: &crate::Module,
        constructor: WrappedConstructor,
    ) -> BackendResult {
        use crate::back::INDENT;

        const ARGUMENT_VARIABLE_NAME: &str = "arg";
        const RETURN_VARIABLE_NAME: &str = "ret";

        // Write function return type and name
        if let crate::TypeInner::Array { base, size, .. } = module.types[constructor.ty].inner {
            write!(self.out, "typedef ")?;
            self.write_type(module, constructor.ty)?;
            write!(self.out, " ret_")?;
            self.write_wrapped_constructor_function_name(module, constructor)?;
            self.write_array_size(module, base, size)?;
            writeln!(self.out, ";")?;

            write!(self.out, "ret_")?;
            self.write_wrapped_constructor_function_name(module, constructor)?;
        } else {
            self.write_type(module, constructor.ty)?;
        }
        write!(self.out, " ")?;
        self.write_wrapped_constructor_function_name(module, constructor)?;

        // Write function parameters
        write!(self.out, "(")?;

        let mut write_arg = |i, ty| -> BackendResult {
            if i != 0 {
                write!(self.out, ", ")?;
            }
            self.write_type(module, ty)?;
            write!(self.out, " {}{}", ARGUMENT_VARIABLE_NAME, i)?;
            if let crate::TypeInner::Array { base, size, .. } = module.types[ty].inner {
                self.write_array_size(module, base, size)?;
            }
            Ok(())
        };

        match module.types[constructor.ty].inner {
            crate::TypeInner::Struct { ref members, .. } => {
                for (i, member) in members.iter().enumerate() {
                    write_arg(i, member.ty)?;
                }
            }
            crate::TypeInner::Array {
                base,
                size: crate::ArraySize::Constant(size),
                ..
            } => {
                let count = module.constants[size].to_array_length().unwrap();
                for i in 0..count as usize {
                    write_arg(i, base)?;
                }
            }
            _ => unreachable!(),
        };

        write!(self.out, ")")?;

        // Write function body
        writeln!(self.out, " {{")?;

        match module.types[constructor.ty].inner {
            crate::TypeInner::Struct { ref members, .. } => {
                let struct_name = &self.names[&NameKey::Type(constructor.ty)];
                writeln!(
                    self.out,
                    "{}{} {} = ({})0;",
                    INDENT, struct_name, RETURN_VARIABLE_NAME, struct_name
                )?;
                for (i, member) in members.iter().enumerate() {
                    let field_name = &self.names[&NameKey::StructMember(constructor.ty, i as u32)];

                    match module.types[member.ty].inner {
                        crate::TypeInner::Matrix {
                            columns,
                            rows: crate::VectorSize::Bi,
                            ..
                        } if member.binding.is_none() => {
                            for j in 0..columns as u8 {
                                writeln!(
                                    self.out,
                                    "{}{}.{}_{} = {}{}[{}];",
                                    INDENT,
                                    RETURN_VARIABLE_NAME,
                                    field_name,
                                    j,
                                    ARGUMENT_VARIABLE_NAME,
                                    i,
                                    j
                                )?;
                            }
                        }
                        ref other => {
                            // We cast arrays of native HLSL `floatCx2`s to arrays of `matCx2`s
                            // (where the inner matrix is represented by a struct with C `float2` members).
                            // See the module-level block comment in mod.rs for details.
                            if let Some(super::writer::MatrixType {
                                columns,
                                rows: crate::VectorSize::Bi,
                                width: 4,
                            }) = super::writer::get_inner_matrix_data(module, member.ty)
                            {
                                write!(
                                    self.out,
                                    "{}{}.{} = (__mat{}x2",
                                    INDENT, RETURN_VARIABLE_NAME, field_name, columns as u8
                                )?;
                                if let crate::TypeInner::Array { base, size, .. } = *other {
                                    self.write_array_size(module, base, size)?;
                                }
                                writeln!(self.out, "){}{};", ARGUMENT_VARIABLE_NAME, i,)?;
                            } else {
                                writeln!(
                                    self.out,
                                    "{}{}.{} = {}{};",
                                    INDENT,
                                    RETURN_VARIABLE_NAME,
                                    field_name,
                                    ARGUMENT_VARIABLE_NAME,
                                    i,
                                )?;
                            }
                        }
                    }
                }
            }
            crate::TypeInner::Array {
                base,
                size: crate::ArraySize::Constant(size),
                ..
            } => {
                write!(self.out, "{}", INDENT)?;
                self.write_type(module, base)?;
                write!(self.out, " {}", RETURN_VARIABLE_NAME)?;
                self.write_array_size(module, base, crate::ArraySize::Constant(size))?;
                write!(self.out, " = {{ ")?;
                let count = module.constants[size].to_array_length().unwrap();
                for i in 0..count {
                    if i != 0 {
                        write!(self.out, ", ")?;
                    }
                    write!(self.out, "{}{}", ARGUMENT_VARIABLE_NAME, i)?;
                }
                writeln!(self.out, " }};",)?;
            }
            _ => unreachable!(),
        }

        // Write return value
        writeln!(self.out, "{}return {};", INDENT, RETURN_VARIABLE_NAME)?;

        // End of function body
        writeln!(self.out, "}}")?;
        // Write extra new line
        writeln!(self.out)?;

        Ok(())
    }

    pub(super) fn write_wrapped_struct_matrix_get_function_name(
        &mut self,
        access: WrappedStructMatrixAccess,
    ) -> BackendResult {
        let name = &self.names[&NameKey::Type(access.ty)];
        let field_name = &self.names[&NameKey::StructMember(access.ty, access.index)];
        write!(self.out, "GetMat{}On{}", field_name, name)?;
        Ok(())
    }

    /// Writes a function used to get a matCx2 from within a structure.
    pub(super) fn write_wrapped_struct_matrix_get_function(
        &mut self,
        module: &crate::Module,
        access: WrappedStructMatrixAccess,
    ) -> BackendResult {
        use crate::back::INDENT;

        const STRUCT_ARGUMENT_VARIABLE_NAME: &str = "obj";

        // Write function return type and name
        let member = match module.types[access.ty].inner {
            crate::TypeInner::Struct { ref members, .. } => &members[access.index as usize],
            _ => unreachable!(),
        };
        let ret_ty = &module.types[member.ty].inner;
        self.write_value_type(module, ret_ty)?;
        write!(self.out, " ")?;
        self.write_wrapped_struct_matrix_get_function_name(access)?;

        // Write function parameters
        write!(self.out, "(")?;
        let struct_name = &self.names[&NameKey::Type(access.ty)];
        write!(
            self.out,
            "{} {}",
            struct_name, STRUCT_ARGUMENT_VARIABLE_NAME
        )?;

        // Write function body
        writeln!(self.out, ") {{")?;

        // Write return value
        write!(self.out, "{}return ", INDENT)?;
        self.write_value_type(module, ret_ty)?;
        write!(self.out, "(")?;
        let field_name = &self.names[&NameKey::StructMember(access.ty, access.index)];
        match module.types[member.ty].inner {
            crate::TypeInner::Matrix { columns, .. } => {
                for i in 0..columns as u8 {
                    if i != 0 {
                        write!(self.out, ", ")?;
                    }
                    write!(
                        self.out,
                        "{}.{}_{}",
                        STRUCT_ARGUMENT_VARIABLE_NAME, field_name, i
                    )?;
                }
            }
            _ => unreachable!(),
        }
        writeln!(self.out, ");")?;

        // End of function body
        writeln!(self.out, "}}")?;
        // Write extra new line
        writeln!(self.out)?;

        Ok(())
    }

    pub(super) fn write_wrapped_struct_matrix_set_function_name(
        &mut self,
        access: WrappedStructMatrixAccess,
    ) -> BackendResult {
        let name = &self.names[&NameKey::Type(access.ty)];
        let field_name = &self.names[&NameKey::StructMember(access.ty, access.index)];
        write!(self.out, "SetMat{}On{}", field_name, name)?;
        Ok(())
    }

    /// Writes a function used to set a matCx2 from within a structure.
    pub(super) fn write_wrapped_struct_matrix_set_function(
        &mut self,
        module: &crate::Module,
        access: WrappedStructMatrixAccess,
    ) -> BackendResult {
        use crate::back::INDENT;

        const STRUCT_ARGUMENT_VARIABLE_NAME: &str = "obj";
        const MATRIX_ARGUMENT_VARIABLE_NAME: &str = "mat";

        // Write function return type and name
        write!(self.out, "void ")?;
        self.write_wrapped_struct_matrix_set_function_name(access)?;

        // Write function parameters
        write!(self.out, "(")?;
        let struct_name = &self.names[&NameKey::Type(access.ty)];
        write!(
            self.out,
            "{} {}, ",
            struct_name, STRUCT_ARGUMENT_VARIABLE_NAME
        )?;
        let member = match module.types[access.ty].inner {
            crate::TypeInner::Struct { ref members, .. } => &members[access.index as usize],
            _ => unreachable!(),
        };
        self.write_type(module, member.ty)?;
        write!(self.out, " {}", MATRIX_ARGUMENT_VARIABLE_NAME)?;
        // Write function body
        writeln!(self.out, ") {{")?;

        let field_name = &self.names[&NameKey::StructMember(access.ty, access.index)];

        match module.types[member.ty].inner {
            crate::TypeInner::Matrix { columns, .. } => {
                for i in 0..columns as u8 {
                    writeln!(
                        self.out,
                        "{}{}.{}_{} = {}[{}];",
                        INDENT,
                        STRUCT_ARGUMENT_VARIABLE_NAME,
                        field_name,
                        i,
                        MATRIX_ARGUMENT_VARIABLE_NAME,
                        i
                    )?;
                }
            }
            _ => unreachable!(),
        }

        // End of function body
        writeln!(self.out, "}}")?;
        // Write extra new line
        writeln!(self.out)?;

        Ok(())
    }

    pub(super) fn write_wrapped_struct_matrix_set_vec_function_name(
        &mut self,
        access: WrappedStructMatrixAccess,
    ) -> BackendResult {
        let name = &self.names[&NameKey::Type(access.ty)];
        let field_name = &self.names[&NameKey::StructMember(access.ty, access.index)];
        write!(self.out, "SetMatVec{}On{}", field_name, name)?;
        Ok(())
    }

    /// Writes a function used to set a vec2 on a matCx2 from within a structure.
    pub(super) fn write_wrapped_struct_matrix_set_vec_function(
        &mut self,
        module: &crate::Module,
        access: WrappedStructMatrixAccess,
    ) -> BackendResult {
        use crate::back::INDENT;

        const STRUCT_ARGUMENT_VARIABLE_NAME: &str = "obj";
        const VECTOR_ARGUMENT_VARIABLE_NAME: &str = "vec";
        const MATRIX_INDEX_ARGUMENT_VARIABLE_NAME: &str = "mat_idx";

        // Write function return type and name
        write!(self.out, "void ")?;
        self.write_wrapped_struct_matrix_set_vec_function_name(access)?;

        // Write function parameters
        write!(self.out, "(")?;
        let struct_name = &self.names[&NameKey::Type(access.ty)];
        write!(
            self.out,
            "{} {}, ",
            struct_name, STRUCT_ARGUMENT_VARIABLE_NAME
        )?;
        let member = match module.types[access.ty].inner {
            crate::TypeInner::Struct { ref members, .. } => &members[access.index as usize],
            _ => unreachable!(),
        };
        let vec_ty = match module.types[member.ty].inner {
            crate::TypeInner::Matrix { rows, width, .. } => crate::TypeInner::Vector {
                size: rows,
                kind: crate::ScalarKind::Float,
                width,
            },
            _ => unreachable!(),
        };
        self.write_value_type(module, &vec_ty)?;
        write!(
            self.out,
            " {}, uint {}",
            VECTOR_ARGUMENT_VARIABLE_NAME, MATRIX_INDEX_ARGUMENT_VARIABLE_NAME
        )?;

        // Write function body
        writeln!(self.out, ") {{")?;

        writeln!(
            self.out,
            "{}switch({}) {{",
            INDENT, MATRIX_INDEX_ARGUMENT_VARIABLE_NAME
        )?;

        let field_name = &self.names[&NameKey::StructMember(access.ty, access.index)];

        match module.types[member.ty].inner {
            crate::TypeInner::Matrix { columns, .. } => {
                for i in 0..columns as u8 {
                    writeln!(
                        self.out,
                        "{}case {}: {{ {}.{}_{} = {}; break; }}",
                        INDENT,
                        i,
                        STRUCT_ARGUMENT_VARIABLE_NAME,
                        field_name,
                        i,
                        VECTOR_ARGUMENT_VARIABLE_NAME
                    )?;
                }
            }
            _ => unreachable!(),
        }

        writeln!(self.out, "{}}}", INDENT)?;

        // End of function body
        writeln!(self.out, "}}")?;
        // Write extra new line
        writeln!(self.out)?;

        Ok(())
    }

    pub(super) fn write_wrapped_struct_matrix_set_scalar_function_name(
        &mut self,
        access: WrappedStructMatrixAccess,
    ) -> BackendResult {
        let name = &self.names[&NameKey::Type(access.ty)];
        let field_name = &self.names[&NameKey::StructMember(access.ty, access.index)];
        write!(self.out, "SetMatScalar{}On{}", field_name, name)?;
        Ok(())
    }

    /// Writes a function used to set a float on a matCx2 from within a structure.
    pub(super) fn write_wrapped_struct_matrix_set_scalar_function(
        &mut self,
        module: &crate::Module,
        access: WrappedStructMatrixAccess,
    ) -> BackendResult {
        use crate::back::INDENT;

        const STRUCT_ARGUMENT_VARIABLE_NAME: &str = "obj";
        const SCALAR_ARGUMENT_VARIABLE_NAME: &str = "scalar";
        const MATRIX_INDEX_ARGUMENT_VARIABLE_NAME: &str = "mat_idx";
        const VECTOR_INDEX_ARGUMENT_VARIABLE_NAME: &str = "vec_idx";

        // Write function return type and name
        write!(self.out, "void ")?;
        self.write_wrapped_struct_matrix_set_scalar_function_name(access)?;

        // Write function parameters
        write!(self.out, "(")?;
        let struct_name = &self.names[&NameKey::Type(access.ty)];
        write!(
            self.out,
            "{} {}, ",
            struct_name, STRUCT_ARGUMENT_VARIABLE_NAME
        )?;
        let member = match module.types[access.ty].inner {
            crate::TypeInner::Struct { ref members, .. } => &members[access.index as usize],
            _ => unreachable!(),
        };
        let scalar_ty = match module.types[member.ty].inner {
            crate::TypeInner::Matrix { width, .. } => crate::TypeInner::Scalar {
                kind: crate::ScalarKind::Float,
                width,
            },
            _ => unreachable!(),
        };
        self.write_value_type(module, &scalar_ty)?;
        write!(
            self.out,
            " {}, uint {}, uint {}",
            SCALAR_ARGUMENT_VARIABLE_NAME,
            MATRIX_INDEX_ARGUMENT_VARIABLE_NAME,
            VECTOR_INDEX_ARGUMENT_VARIABLE_NAME
        )?;

        // Write function body
        writeln!(self.out, ") {{")?;

        writeln!(
            self.out,
            "{}switch({}) {{",
            INDENT, MATRIX_INDEX_ARGUMENT_VARIABLE_NAME
        )?;

        let field_name = &self.names[&NameKey::StructMember(access.ty, access.index)];

        match module.types[member.ty].inner {
            crate::TypeInner::Matrix { columns, .. } => {
                for i in 0..columns as u8 {
                    writeln!(
                        self.out,
                        "{}case {}: {{ {}.{}_{}[{}] = {}; break; }}",
                        INDENT,
                        i,
                        STRUCT_ARGUMENT_VARIABLE_NAME,
                        field_name,
                        i,
                        VECTOR_INDEX_ARGUMENT_VARIABLE_NAME,
                        SCALAR_ARGUMENT_VARIABLE_NAME
                    )?;
                }
            }
            _ => unreachable!(),
        }

        writeln!(self.out, "{}}}", INDENT)?;

        // End of function body
        writeln!(self.out, "}}")?;
        // Write extra new line
        writeln!(self.out)?;

        Ok(())
    }

    /// Helper function that write wrapped function for `Expression::ImageQuery` and `Expression::ArrayLength`
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/dx-graphics-hlsl-to-getdimensions>
    pub(super) fn write_wrapped_functions(
        &mut self,
        module: &crate::Module,
        func_ctx: &FunctionCtx,
    ) -> BackendResult {
        for (handle, _) in func_ctx.expressions.iter() {
            match func_ctx.expressions[handle] {
                crate::Expression::ArrayLength(expr) => {
                    let global_expr = match func_ctx.expressions[expr] {
                        crate::Expression::GlobalVariable(_) => expr,
                        crate::Expression::AccessIndex { base, index: _ } => base,
                        ref other => unreachable!("Array length of {:?}", other),
                    };
                    let global_var = match func_ctx.expressions[global_expr] {
                        crate::Expression::GlobalVariable(var_handle) => {
                            &module.global_variables[var_handle]
                        }
                        ref other => unreachable!("Array length of base {:?}", other),
                    };
                    let storage_access = match global_var.space {
                        crate::AddressSpace::Storage { access } => access,
                        _ => crate::StorageAccess::default(),
                    };
                    let wal = WrappedArrayLength {
                        writable: storage_access.contains(crate::StorageAccess::STORE),
                    };

                    if !self.wrapped.array_lengths.contains(&wal) {
                        self.write_wrapped_array_length_function(module, wal, handle, func_ctx)?;
                        self.wrapped.array_lengths.insert(wal);
                    }
                }
                crate::Expression::ImageQuery { image, query } => {
                    let wiq = match *func_ctx.info[image].ty.inner_with(&module.types) {
                        crate::TypeInner::Image {
                            dim,
                            arrayed,
                            class,
                        } => WrappedImageQuery {
                            dim,
                            arrayed,
                            class,
                            query: query.into(),
                        },
                        _ => unreachable!("we only query images"),
                    };

                    if !self.wrapped.image_queries.contains(&wiq) {
                        self.write_wrapped_image_query_function(module, wiq, handle, func_ctx)?;
                        self.wrapped.image_queries.insert(wiq);
                    }
                }
                // Write `WrappedConstructor` for structs that are loaded from `AddressSpace::Storage`
                // since they will later be used by the fn `write_storage_load`
                crate::Expression::Load { pointer } => {
                    let pointer_space = func_ctx.info[pointer]
                        .ty
                        .inner_with(&module.types)
                        .pointer_space();

                    if let Some(crate::AddressSpace::Storage { .. }) = pointer_space {
                        if let Some(ty) = func_ctx.info[handle].ty.handle() {
                            write_wrapped_constructor(self, ty, module, func_ctx)?;
                        }
                    }

                    fn write_wrapped_constructor<W: Write>(
                        writer: &mut super::Writer<'_, W>,
                        ty: Handle<crate::Type>,
                        module: &crate::Module,
                        func_ctx: &FunctionCtx,
                    ) -> BackendResult {
                        match module.types[ty].inner {
                            crate::TypeInner::Struct { ref members, .. } => {
                                for member in members {
                                    write_wrapped_constructor(writer, member.ty, module, func_ctx)?;
                                }

                                let constructor = WrappedConstructor { ty };
                                if !writer.wrapped.constructors.contains(&constructor) {
                                    writer
                                        .write_wrapped_constructor_function(module, constructor)?;
                                    writer.wrapped.constructors.insert(constructor);
                                }
                            }
                            crate::TypeInner::Array { base, .. } => {
                                write_wrapped_constructor(writer, base, module, func_ctx)?;
                            }
                            _ => {}
                        };

                        Ok(())
                    }
                }
                crate::Expression::Compose { ty, components: _ } => {
                    let constructor = match module.types[ty].inner {
                        crate::TypeInner::Struct { .. } | crate::TypeInner::Array { .. } => {
                            WrappedConstructor { ty }
                        }
                        _ => continue,
                    };
                    if !self.wrapped.constructors.contains(&constructor) {
                        self.write_wrapped_constructor_function(module, constructor)?;
                        self.wrapped.constructors.insert(constructor);
                    }
                }
                // We treat matrices of the form `matCx2` as a sequence of C `vec2`s
                // (see top level module docs for details).
                //
                // The functions injected here are required to get the matrix accesses working.
                crate::Expression::AccessIndex { base, index } => {
                    let base_ty_res = &func_ctx.info[base].ty;
                    let mut resolved = base_ty_res.inner_with(&module.types);
                    let base_ty_handle = match *resolved {
                        crate::TypeInner::Pointer { base, .. } => {
                            resolved = &module.types[base].inner;
                            Some(base)
                        }
                        _ => base_ty_res.handle(),
                    };
                    if let crate::TypeInner::Struct { ref members, .. } = *resolved {
                        let member = &members[index as usize];

                        match module.types[member.ty].inner {
                            crate::TypeInner::Matrix {
                                rows: crate::VectorSize::Bi,
                                ..
                            } if member.binding.is_none() => {
                                let ty = base_ty_handle.unwrap();
                                let access = WrappedStructMatrixAccess { ty, index };

                                if !self.wrapped.struct_matrix_access.contains(&access) {
                                    self.write_wrapped_struct_matrix_get_function(module, access)?;
                                    self.write_wrapped_struct_matrix_set_function(module, access)?;
                                    self.write_wrapped_struct_matrix_set_vec_function(
                                        module, access,
                                    )?;
                                    self.write_wrapped_struct_matrix_set_scalar_function(
                                        module, access,
                                    )?;
                                    self.wrapped.struct_matrix_access.insert(access);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            };
        }

        Ok(())
    }

    pub(super) fn write_wrapped_constructor_function_for_constant(
        &mut self,
        module: &crate::Module,
        constant: &crate::Constant,
    ) -> BackendResult {
        if let crate::ConstantInner::Composite { ty, ref components } = constant.inner {
            match module.types[ty].inner {
                crate::TypeInner::Struct { .. } | crate::TypeInner::Array { .. } => {
                    let constructor = WrappedConstructor { ty };
                    if !self.wrapped.constructors.contains(&constructor) {
                        self.write_wrapped_constructor_function(module, constructor)?;
                        self.wrapped.constructors.insert(constructor);
                    }
                }
                _ => {}
            }
            for constant in components {
                self.write_wrapped_constructor_function_for_constant(
                    module,
                    &module.constants[*constant],
                )?;
            }
        }

        Ok(())
    }

    pub(super) fn write_texture_coordinates(
        &mut self,
        kind: &str,
        coordinate: Handle<crate::Expression>,
        array_index: Option<Handle<crate::Expression>>,
        mip_level: Option<Handle<crate::Expression>>,
        module: &crate::Module,
        func_ctx: &FunctionCtx,
    ) -> BackendResult {
        // HLSL expects the array index to be merged with the coordinate
        let extra = array_index.is_some() as usize + (mip_level.is_some()) as usize;
        if extra == 0 {
            self.write_expr(module, coordinate, func_ctx)?;
        } else {
            let num_coords = match *func_ctx.info[coordinate].ty.inner_with(&module.types) {
                crate::TypeInner::Scalar { .. } => 1,
                crate::TypeInner::Vector { size, .. } => size as usize,
                _ => unreachable!(),
            };
            write!(self.out, "{}{}(", kind, num_coords + extra)?;
            self.write_expr(module, coordinate, func_ctx)?;
            if let Some(expr) = array_index {
                write!(self.out, ", ")?;
                self.write_expr(module, expr, func_ctx)?;
            }
            if let Some(expr) = mip_level {
                write!(self.out, ", ")?;
                self.write_expr(module, expr, func_ctx)?;
            }
            write!(self.out, ")")?;
        }
        Ok(())
    }

    pub(super) fn write_mat_cx2_typedef_and_functions(
        &mut self,
        WrappedMatCx2 { columns }: WrappedMatCx2,
    ) -> BackendResult {
        use crate::back::INDENT;

        // typedef
        write!(self.out, "typedef struct {{ ")?;
        for i in 0..columns as u8 {
            write!(self.out, "float2 _{}; ", i)?;
        }
        writeln!(self.out, "}} __mat{}x2;", columns as u8)?;

        // __get_col_of_mat
        writeln!(
            self.out,
            "float2 __get_col_of_mat{}x2(__mat{}x2 mat, uint idx) {{",
            columns as u8, columns as u8
        )?;
        writeln!(self.out, "{}switch(idx) {{", INDENT)?;
        for i in 0..columns as u8 {
            writeln!(self.out, "{}case {}: {{ return mat._{}; }}", INDENT, i, i)?;
        }
        writeln!(self.out, "{}default: {{ return (float2)0; }}", INDENT)?;
        writeln!(self.out, "{}}}", INDENT)?;
        writeln!(self.out, "}}")?;

        // __set_col_of_mat
        writeln!(
            self.out,
            "void __set_col_of_mat{}x2(__mat{}x2 mat, uint idx, float2 value) {{",
            columns as u8, columns as u8
        )?;
        writeln!(self.out, "{}switch(idx) {{", INDENT)?;
        for i in 0..columns as u8 {
            writeln!(
                self.out,
                "{}case {}: {{ mat._{} = value; break; }}",
                INDENT, i, i
            )?;
        }
        writeln!(self.out, "{}}}", INDENT)?;
        writeln!(self.out, "}}")?;

        // __set_el_of_mat
        writeln!(
            self.out,
            "void __set_el_of_mat{}x2(__mat{}x2 mat, uint idx, uint vec_idx, float value) {{",
            columns as u8, columns as u8
        )?;
        writeln!(self.out, "{}switch(idx) {{", INDENT)?;
        for i in 0..columns as u8 {
            writeln!(
                self.out,
                "{}case {}: {{ mat._{}[vec_idx] = value; break; }}",
                INDENT, i, i
            )?;
        }
        writeln!(self.out, "{}}}", INDENT)?;
        writeln!(self.out, "}}")?;

        writeln!(self.out)?;

        Ok(())
    }

    pub(super) fn write_all_mat_cx2_typedefs_and_functions(
        &mut self,
        module: &crate::Module,
    ) -> BackendResult {
        for (handle, _) in module.global_variables.iter() {
            let global = &module.global_variables[handle];

            if global.space == crate::AddressSpace::Uniform {
                if let Some(super::writer::MatrixType {
                    columns,
                    rows: crate::VectorSize::Bi,
                    width: 4,
                }) = super::writer::get_inner_matrix_data(module, global.ty)
                {
                    let entry = WrappedMatCx2 { columns };
                    if !self.wrapped.mat_cx2s.contains(&entry) {
                        self.write_mat_cx2_typedef_and_functions(entry)?;
                        self.wrapped.mat_cx2s.insert(entry);
                    }
                }
            }
        }

        for (_, ty) in module.types.iter() {
            if let crate::TypeInner::Struct { ref members, .. } = ty.inner {
                for member in members.iter() {
                    if let crate::TypeInner::Array { .. } = module.types[member.ty].inner {
                        if let Some(super::writer::MatrixType {
                            columns,
                            rows: crate::VectorSize::Bi,
                            width: 4,
                        }) = super::writer::get_inner_matrix_data(module, member.ty)
                        {
                            let entry = WrappedMatCx2 { columns };
                            if !self.wrapped.mat_cx2s.contains(&entry) {
                                self.write_mat_cx2_typedef_and_functions(entry)?;
                                self.wrapped.mat_cx2s.insert(entry);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
