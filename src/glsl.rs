use glsl::syntax::NonEmpty;

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
            crate::ImageFormat::RGB10A2_UNorm => "rgb10a2",
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
    pub fn to_declaration(&self, set_id: u32) -> glsl::syntax::Declaration {
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
            | crate::DescriptorType::AccelerationStructure => {
                type_qualifier
                    .qualifiers
                    .push(glsl::syntax::TypeQualifierSpec::Storage(
                        glsl::syntax::StorageQualifier::Uniform,
                    ));
            }
        }

        let type_specifier = match self.descriptor_type {
            crate::DescriptorType::Sampler => todo!(),
            crate::DescriptorType::StorageImage { .. } => {
                glsl::syntax::TypeSpecifierNonArray::Image2D
            }
            crate::DescriptorType::SampledImage => {
                glsl::syntax::TypeSpecifierNonArray::Sampler2D
            }
            crate::DescriptorType::AccelerationStructure => {
                glsl::syntax::TypeSpecifierNonArray::TypeName("accelerationStructureEXT".into())
            }
        };

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
    pub fn to_declarations(
        &self,
        set_id: u32,
    ) -> impl ExactSizeIterator<Item = glsl::syntax::Declaration> + '_ {
        self.bindings
            .iter()
            .map(move |binding| binding.to_declaration(set_id))
    }
}
