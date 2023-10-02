use glsl::syntax::NonEmpty;

use crate::PlayoutModule;
use inflector::Inflector;

impl crate::ImageFormat {
    pub fn to_layout_qualifier(&self) -> &'static str {
        match self {
            crate::ImageFormat::RGBA32_Float => "rgba32f",
            crate::ImageFormat::RGBA16_Float => "rgba16f",
            crate::ImageFormat::RG32_Float => "rg32f",
            crate::ImageFormat::RG16_Float => "rg16f",
            crate::ImageFormat::R11G11B10_Float => "r11f_g11f_b10f",
            crate::ImageFormat::R32_Float => "r32f",
            crate::ImageFormat::R16_Float => "r16f",
            crate::ImageFormat::RGBA16_UNorm => "rgba16",
            crate::ImageFormat::RGB10A2_UNorm => "rgb10_a2",
            crate::ImageFormat::RBGA8_UNorm => "rgba8",
            crate::ImageFormat::RG16_UNorm => "rg16",
            crate::ImageFormat::RG8_UNorm => "rg8",
            crate::ImageFormat::R16_UNorm => "r16",
            crate::ImageFormat::R8_UNorm => "r8",
            crate::ImageFormat::RGBA16_SNorm => "rgba16_snorm",
            crate::ImageFormat::RBGA8_SNorm => "rgba8_snorm",
            crate::ImageFormat::RG16_SNorm => "rg16_snorm",
            crate::ImageFormat::RG8_SNorm => "rg8_snorm",
            crate::ImageFormat::R16_SNorm => "r16_snorm",
            crate::ImageFormat::R8_SNorm => "r8_snorm",
            crate::ImageFormat::RGBA32_SInt => "rgba32i",
            crate::ImageFormat::RGBA16_SInt => "rgba16i",
            crate::ImageFormat::RGBA8_SInt => "rgba8i",
            crate::ImageFormat::RG32_SInt => "rg32i",
            crate::ImageFormat::RG16_SInt => "rg16i",
            crate::ImageFormat::RG8_SInt => "rg8i",
            crate::ImageFormat::R32_SInt => "r32i",
            crate::ImageFormat::R16_SInt => "r16i",
            crate::ImageFormat::R8_SInt => "r8i",
            crate::ImageFormat::RGBA32_UInt => "rgba32ui",
            crate::ImageFormat::RGBA16_UInt => "rgba16ui",
            crate::ImageFormat::RGB10A2_UInt => "rgb10a2ui",
            crate::ImageFormat::RGBA8_UInt => "rgba8ui",
            crate::ImageFormat::RG32_UInt => "rg32ui",
            crate::ImageFormat::RG16_UInt => "rg16ui",
            crate::ImageFormat::RG8_UInt => "rg8ui",
            crate::ImageFormat::R32_UInt => "r32ui",
            crate::ImageFormat::R16_UInt => "r16ui",
            crate::ImageFormat::R8_UInt => "r8ui",
        }
    }
}

impl crate::Binding {
    pub fn to_declaration(&self, module: &PlayoutModule, set_id: u32) -> glsl::syntax::Declaration {
        let mut layout_qualifier = glsl::syntax::LayoutQualifier {
            ids: NonEmpty::from_non_empty_iter([
                glsl::syntax::LayoutQualifierSpec::Identifier(
                    "binding".into(),
                    Some(Box::new(glsl::syntax::Expr::UIntConst(self.binding))),
                ),
                glsl::syntax::LayoutQualifierSpec::Identifier(
                    "set".into(),
                    Some(Box::new(glsl::syntax::Expr::UIntConst(set_id))),
                ),
            ])
            .unwrap(),
        };

        if let crate::DescriptorType::StorageImage { format } = self.descriptor_type {
            layout_qualifier
                .ids
                .push(glsl::syntax::LayoutQualifierSpec::Identifier(
                    format.to_layout_qualifier().into(),
                    None,
                ));
        }

        let mut type_qualifier = glsl::syntax::TypeQualifier {
            qualifiers: NonEmpty::from_non_empty_iter([glsl::syntax::TypeQualifierSpec::Layout(
                layout_qualifier,
            )])
            .unwrap(),
        };

        match self.descriptor_type {
            crate::DescriptorType::Sampler => todo!(),
            crate::DescriptorType::StorageImage { .. }
            | crate::DescriptorType::SampledImage
            | crate::DescriptorType::AccelerationStructure
            | crate::DescriptorType::UniformBuffer { .. } => {
                type_qualifier
                    .qualifiers
                    .push(glsl::syntax::TypeQualifierSpec::Storage(
                        glsl::syntax::StorageQualifier::Uniform,
                    ));
            }
            crate::DescriptorType::StorageBuffer { .. } => {
                type_qualifier
                    .qualifiers
                    .push(glsl::syntax::TypeQualifierSpec::Storage(
                        glsl::syntax::StorageQualifier::Buffer,
                    ));
            }
        }

        let array_specifier = if self.descriptor_count > 1 {
            Some(glsl::syntax::ArraySpecifier {
                dimensions: NonEmpty::from_non_empty_iter([
                    glsl::syntax::ArraySpecifierDimension::ExplicitlySized(Box::new(
                        glsl::syntax::Expr::UIntConst(self.descriptor_count),
                    )),
                ])
                .unwrap(),
            })
        } else {
            None
        };

        let type_specifier = match &self.descriptor_type {
            crate::DescriptorType::Sampler => todo!(),
            crate::DescriptorType::StorageImage { format } => {
                use crate::ImageFormatDataMode::*;
                match format.data_mode() {
                    Float | UNorm | SNorm => glsl::syntax::TypeSpecifierNonArray::Image2D,
                    crate::ImageFormatDataMode::SInt => {
                        glsl::syntax::TypeSpecifierNonArray::IImage2D
                    }
                    crate::ImageFormatDataMode::UInt => {
                        glsl::syntax::TypeSpecifierNonArray::UImage2D
                    }
                }
            }
            crate::DescriptorType::SampledImage => glsl::syntax::TypeSpecifierNonArray::Sampler2D,
            crate::DescriptorType::AccelerationStructure => {
                glsl::syntax::TypeSpecifierNonArray::TypeName("accelerationStructureEXT".into())
            }
            crate::DescriptorType::UniformBuffer { ty }
            | crate::DescriptorType::StorageBuffer { ty } => {
                let (fields, identifier): (
                    Vec<glsl::syntax::StructFieldSpecifier>,
                    Option<glsl::syntax::ArrayedIdentifier>,
                ) = match ty {
                    crate::Type::Path(path) => {
                        // Specific optimization for when the type directly references a struct.
                        // Take that struct and flatten it out directly as a uniform/storage block.
                        let fields = module
                            .data_structs
                            .get(&path.get_ident().unwrap().to_string())
                            .unwrap()
                            .fields
                            .iter()
                            .map(|field| field.to_field())
                            .collect();
                        let identifier = Some(glsl::syntax::ArrayedIdentifier {
                            ident: self.ident.as_str().into(),
                            array_spec: array_specifier,
                        });
                        (fields, identifier)
                    }
                    _ => (vec![ty.as_field(self.ident.as_str())], None),
                };
                return glsl::syntax::Declaration::Block(glsl::syntax::Block {
                    qualifier: type_qualifier,
                    name: self.ident.to_pascal_case().into(),
                    fields,
                    identifier,
                });
            }
        };

        glsl::syntax::Declaration::InitDeclaratorList(glsl::syntax::InitDeclaratorList {
            head: glsl::syntax::SingleDeclaration {
                ty: glsl::syntax::FullySpecifiedType {
                    qualifier: Some(type_qualifier),
                    ty: glsl::syntax::TypeSpecifier {
                        ty: type_specifier,
                        array_specifier: None,
                    },
                },
                name: Some(self.ident.as_str().into()),
                array_specifier,
                initializer: None,
            },
            tail: Vec::new(),
        })
    }
}

impl crate::SetLayout {
    pub fn to_declarations<'a>(
        &'a self,
        module: &'a PlayoutModule,
    ) -> impl ExactSizeIterator<Item = glsl::syntax::Declaration> + 'a {
        let set_id = self.set;
        self.bindings
            .iter()
            .map(move |binding| binding.to_declaration(module, set_id))
    }
}

impl crate::PlayoutModule {
    pub fn show(&self, writer: &mut impl std::fmt::Write) {
        for (_, data_struct) in self.data_structs.iter() {
            glsl::transpiler::glsl::show_struct(writer, &data_struct.to_struct_specifier());
        }
        for decl in self
            .descriptor_sets
            .iter()
            .flat_map(|set| set.to_declarations(self))
        {
            glsl::transpiler::glsl::show_declaration(writer, &decl);
        }
    }
}

impl crate::Field {
    pub fn to_field(&self) -> glsl::syntax::StructFieldSpecifier {
        self.ty.as_field(self.ident.as_ref().unwrap().as_str())
    }
}

impl crate::Type {
    pub fn base_type(&self) -> glsl::syntax::TypeSpecifierNonArray {
        use crate::Type::*;
        match self {
            Primitive(ty) => ty.to_type_specifier_non_array(),
            Array { ty, .. } => ty.base_type(),
            Slice { ty } => ty.base_type(),
            Path(path) => glsl::syntax::TypeSpecifierNonArray::TypeName(
                path.get_ident().unwrap().to_string().into(),
            ),
        }
    }
    pub fn as_field(&self, ident: &str) -> glsl::syntax::StructFieldSpecifier {
        glsl::syntax::StructFieldSpecifier {
            qualifier: None,
            ty: glsl::syntax::TypeSpecifier {
                ty: match self {
                    crate::Type::Path(path) => glsl::syntax::TypeSpecifierNonArray::TypeName(
                        path.get_ident().unwrap().to_string().into(),
                    ),
                    _ => self.base_type(),
                },
                array_specifier: None,
            },
            identifiers: NonEmpty::from_non_empty_iter([glsl::syntax::ArrayedIdentifier {
                ident: ident.into(),
                array_spec: match self {
                    crate::Type::Array { size, .. } => Some(glsl::syntax::ArraySpecifier {
                        dimensions: NonEmpty::from_non_empty_iter([
                            glsl::syntax::ArraySpecifierDimension::ExplicitlySized(Box::new(
                                glsl::syntax::Expr::UIntConst(*size as u32),
                            )),
                        ])
                        .unwrap(),
                    }),
                    crate::Type::Primitive(_) => None,
                    crate::Type::Path(_) => None,
                    crate::Type::Slice { .. } => Some(glsl::syntax::ArraySpecifier {
                        dimensions: NonEmpty::from_non_empty_iter([
                            glsl::syntax::ArraySpecifierDimension::Unsized,
                        ])
                        .unwrap(),
                    }),
                },
            }])
            .unwrap(),
        }
    }
}

impl crate::PrimitiveType {
    pub fn to_type_specifier_non_array(&self) -> glsl::syntax::TypeSpecifierNonArray {
        use glsl::syntax::TypeSpecifierNonArray::*;
        match self {
            crate::PrimitiveType::Single(ty) => match ty {
                crate::PrimitiveTypeSingle::U8 => TypeName("uint8_t".into()),
                crate::PrimitiveTypeSingle::U16 => TypeName("uint16_t".into()),
                crate::PrimitiveTypeSingle::U32 => UInt,
                crate::PrimitiveTypeSingle::U64 => TypeName("uint64_t".into()),
                crate::PrimitiveTypeSingle::I8 => TypeName("int8_t".into()),
                crate::PrimitiveTypeSingle::I16 => TypeName("int16_t".into()),
                crate::PrimitiveTypeSingle::I32 => Int,
                crate::PrimitiveTypeSingle::I64 => TypeName("int64_t".into()),
                crate::PrimitiveTypeSingle::F16 => TypeName("float16_t".into()),
                crate::PrimitiveTypeSingle::F32 => Float,
                crate::PrimitiveTypeSingle::F64 => Double,
                crate::PrimitiveTypeSingle::Bool => Bool,
            },
            crate::PrimitiveType::Vec { ty, length } => match (ty, length) {
                (crate::PrimitiveTypeSingle::U8, 2) => TypeName("u8vec2".into()),
                (crate::PrimitiveTypeSingle::U8, 3) => TypeName("u8vec3".into()),
                (crate::PrimitiveTypeSingle::U8, 4) => TypeName("u8vec4".into()),
                (crate::PrimitiveTypeSingle::U16, 2) => TypeName("u16vec2".into()),
                (crate::PrimitiveTypeSingle::U16, 3) => TypeName("u16vec3".into()),
                (crate::PrimitiveTypeSingle::U16, 4) => TypeName("u16vec4".into()),
                (crate::PrimitiveTypeSingle::U32, 2) => UVec2,
                (crate::PrimitiveTypeSingle::U32, 3) => UVec3,
                (crate::PrimitiveTypeSingle::U32, 4) => UVec4,
                (crate::PrimitiveTypeSingle::U64, 2) => TypeName("u64vec2".into()),
                (crate::PrimitiveTypeSingle::U64, 3) => TypeName("u64vec3".into()),
                (crate::PrimitiveTypeSingle::U64, 4) => TypeName("u64vec4".into()),

                (crate::PrimitiveTypeSingle::I8, 2) => TypeName("i8vec2".into()),
                (crate::PrimitiveTypeSingle::I8, 3) => TypeName("i8vec3".into()),
                (crate::PrimitiveTypeSingle::I8, 4) => TypeName("i8vec4".into()),
                (crate::PrimitiveTypeSingle::I16, 2) => TypeName("i16vec2".into()),
                (crate::PrimitiveTypeSingle::I16, 3) => TypeName("i16vec3".into()),
                (crate::PrimitiveTypeSingle::I16, 4) => TypeName("i16vec4".into()),
                (crate::PrimitiveTypeSingle::I32, 2) => IVec2,
                (crate::PrimitiveTypeSingle::I32, 3) => IVec3,
                (crate::PrimitiveTypeSingle::I32, 4) => IVec4,
                (crate::PrimitiveTypeSingle::I64, 2) => TypeName("i64vec2".into()),
                (crate::PrimitiveTypeSingle::I64, 3) => TypeName("i64vec3".into()),
                (crate::PrimitiveTypeSingle::I64, 4) => TypeName("i64vec4".into()),

                (crate::PrimitiveTypeSingle::F16, 2) => TypeName("f16vec2".into()),
                (crate::PrimitiveTypeSingle::F16, 3) => TypeName("f16vec3".into()),
                (crate::PrimitiveTypeSingle::F16, 4) => TypeName("f16vec4".into()),
                (crate::PrimitiveTypeSingle::F32, 2) => Vec2,
                (crate::PrimitiveTypeSingle::F32, 3) => Vec3,
                (crate::PrimitiveTypeSingle::F32, 4) => Vec4,
                (crate::PrimitiveTypeSingle::F64, 2) => TypeName("f64vec2".into()),
                (crate::PrimitiveTypeSingle::F64, 3) => TypeName("f64vec3".into()),
                (crate::PrimitiveTypeSingle::F64, 4) => TypeName("f64vec4".into()),

                (crate::PrimitiveTypeSingle::Bool, 2) => BVec2,
                (crate::PrimitiveTypeSingle::Bool, 3) => BVec3,
                (crate::PrimitiveTypeSingle::Bool, 4) => BVec4,
                _ => panic!(),
            },
            crate::PrimitiveType::Mat { ty, rows, columns } => match (ty, rows, columns) {
                (crate::PrimitiveTypeSingle::F16, 2, 2) => TypeName("f16mat2x2".into()),
                (crate::PrimitiveTypeSingle::F16, 3, 3) => TypeName("f16mat3x3".into()),
                (crate::PrimitiveTypeSingle::F16, 4, 4) => TypeName("f16mat4x4".into()),
                (crate::PrimitiveTypeSingle::F32, 2, 2) => Mat2,
                (crate::PrimitiveTypeSingle::F32, 3, 3) => Mat3,
                (crate::PrimitiveTypeSingle::F32, 4, 4) => Mat4,
                (crate::PrimitiveTypeSingle::F64, 2, 2) => TypeName("f64mat2x2".into()),
                (crate::PrimitiveTypeSingle::F64, 3, 3) => TypeName("f64mat3x3".into()),
                (crate::PrimitiveTypeSingle::F64, 4, 4) => TypeName("f64mat4x4".into()),
                _ => panic!(),
            },
        }
    }
}

impl crate::DataStruct {
    pub fn to_struct_specifier(&self) -> glsl::syntax::StructSpecifier {
        let fields = self.fields.iter().map(|field| field.to_field());

        glsl::syntax::StructSpecifier {
            name: Some(self.ident.as_str().into()),
            fields: NonEmpty::from_non_empty_iter(fields).unwrap(),
        }
    }
}
