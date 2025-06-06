// Copyright (c) 2019-present Dmitry Stepanov and Fyrox Engine contributors.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! A general-purpose property editor definition that creates
//! a nested inspector within an [Expander](crate::expander::Expander) widget.
use crate::{
    core::reflect::prelude::*,
    inspector::{
        editors::{
            PropertyEditorBuildContext, PropertyEditorDefinition, PropertyEditorInstance,
            PropertyEditorMessageContext, PropertyEditorTranslationContext,
        },
        make_expander_container, FieldKind, Inspector, InspectorBuilder, InspectorContext,
        InspectorError, InspectorMessage, PropertyChanged,
    },
    message::{MessageDirection, UiMessage},
    widget::WidgetBuilder,
};
use fyrox_core::pool::Handle;
use fyrox_core::PhantomDataSendSync;
use fyrox_graph::BaseSceneGraph;
use std::{
    any::TypeId,
    fmt::{Debug, Formatter},
};

/// A general-purpose property editor definition that creates
/// a nested inspector within an [Expander](crate::expander::Expander) widget to allow the user
/// to edited properties of type T.
/// The expander is labeled with [FieldMetadata::display_name].
/// The layer_index for the inner inspector is increased by 1.
pub struct InspectablePropertyEditorDefinition<T>
where
    T: Reflect,
{
    #[allow(dead_code)]
    phantom: PhantomDataSendSync<T>,
}

impl<T> InspectablePropertyEditorDefinition<T>
where
    T: Reflect,
{
    pub fn new() -> Self {
        Self {
            phantom: Default::default(),
        }
    }
}

impl<T> Debug for InspectablePropertyEditorDefinition<T>
where
    T: Reflect,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "InspectablePropertyEditorDefinition")
    }
}

impl<T> PropertyEditorDefinition for InspectablePropertyEditorDefinition<T>
where
    T: Reflect,
{
    fn value_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn create_instance(
        &self,
        ctx: PropertyEditorBuildContext,
    ) -> Result<PropertyEditorInstance, InspectorError> {
        let value = ctx.property_info.cast_value::<T>()?;

        let inspector_context = InspectorContext::from_object(
            value,
            ctx.build_context,
            ctx.definition_container.clone(),
            ctx.environment.clone(),
            ctx.sync_flag,
            ctx.layer_index + 1,
            ctx.generate_property_string_values,
            ctx.filter,
            ctx.name_column_width,
        );

        let editor;
        let container = make_expander_container(
            ctx.layer_index,
            ctx.property_info.display_name,
            ctx.property_info.description,
            Handle::NONE,
            {
                editor = InspectorBuilder::new(WidgetBuilder::new())
                    .with_context(inspector_context)
                    .build(ctx.build_context);
                editor
            },
            ctx.name_column_width,
            ctx.build_context,
        );

        Ok(PropertyEditorInstance::Custom { container, editor })
    }

    /// Instead of creating a message to update its widget,
    /// call [InspectorContext::sync] to directly send whatever messages are necessary
    /// and return None.
    fn create_message(
        &self,
        ctx: PropertyEditorMessageContext,
    ) -> Result<Option<UiMessage>, InspectorError> {
        let value = ctx.property_info.cast_value::<T>()?;

        let mut error_group = Vec::new();

        let inspector_context = ctx
            .ui
            .node(ctx.instance)
            .cast::<Inspector>()
            .expect("Must be Inspector!")
            .context()
            .clone();
        if let Err(e) = inspector_context.sync(
            value,
            ctx.ui,
            ctx.layer_index + 1,
            ctx.generate_property_string_values,
            ctx.filter,
        ) {
            error_group.extend(e)
        }

        if error_group.is_empty() {
            Ok(None)
        } else {
            Err(InspectorError::Group(error_group))
        }
    }

    fn translate_message(&self, ctx: PropertyEditorTranslationContext) -> Option<PropertyChanged> {
        if let Some(InspectorMessage::PropertyChanged(msg)) = ctx.message.data::<InspectorMessage>()
        {
            if ctx.message.direction() == MessageDirection::FromWidget {
                return Some(PropertyChanged {
                    name: ctx.name.to_owned(),

                    value: FieldKind::Inspectable(Box::new(msg.clone())),
                });
            }
        }

        None
    }
}
