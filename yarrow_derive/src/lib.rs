use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::crate_name;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, spanned::Spanned, DeriveInput, Ident};

fn root() -> proc_macro2::TokenStream {
    match crate_name("yarrow").expect("yarrow not found in Cargo.toml") {
        proc_macro_crate::FoundCrate::Itself => quote! { crate },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { ::#ident }
        }
    }
}

#[proc_macro_attribute]
pub fn element_builder(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident.clone();
    let generics = ast.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let crate_name = root();

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            if let syn::Fields::Named(ref mut fields) = struct_data.fields {
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            /// The z index of the element
                            ///
                            /// If this method is not used, then the current z index from the window context will
                            /// be used.
                            pub z_index: Option<#crate_name::math::ZIndex>
                        })
                        .unwrap(),
                );

                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            /// The ID of the scissoring rectangle this element belongs to.
                            ///
                            /// If this method is not used, then the current scissoring rectangle ID from the
                            /// window context will be used.
                            pub scissor_rect: Option<#crate_name::ScissorRectID>
                        })
                        .unwrap(),
                );
            }

            quote! {
                #ast

                impl #impl_generics #name #ty_generics #where_clause {
                    /// The z index of the element
                    ///
                    /// If this method is not used, then the current z index from the window context will
                    /// be used.
                    pub const fn z_index(mut self, z_index: #crate_name::math::ZIndex) -> Self {
                        self.z_index = Some(z_index);
                        self
                    }

                    /// The ID of the scissoring rectangle this element belongs to.
                    ///
                    /// If this method is not used, then the current scissoring rectangle ID from the
                    /// window context will be used.
                    pub const fn scissor_rect(mut self, scissor_rect: #crate_name::ScissorRectID) -> Self {
                        self.scissor_rect = Some(scissor_rect);
                        self
                    }
                }
            }
            .into()
        }
        _ => syn::Error::new(ast.span(), "`element_builder` has to be used with structs")
            .to_compile_error()
            .into(),
    }
}

#[proc_macro_attribute]
pub fn element_builder_class(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident.clone();
    let generics = ast.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let crate_name = root();

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            if let syn::Fields::Named(ref mut fields) = struct_data.fields {
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            /// The style class ID
                            ///
                            /// If this method is not used, then the current class from the window context will
                            /// be used.
                            pub class: Option<#crate_name::style::ClassID>
                        })
                        .unwrap(),
                );
            }

            quote! {
                #ast

                impl #impl_generics #name #ty_generics #where_clause {
                    /// The style class ID
                    ///
                    /// If this method is not used, then the current class from the window context will
                    /// be used.
                    pub const fn class(mut self, class: #crate_name::style::ClassID) -> Self {
                        self.class = Some(class);
                        self
                    }
                }
            }
            .into()
        }
        _ => syn::Error::new(
            ast.span(),
            "`element_builder_class` has to be used with structs ",
        )
        .to_compile_error()
        .into(),
    }
}

#[proc_macro_attribute]
pub fn element_builder_rect(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident.clone();
    let generics = ast.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let crate_name = root();

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            if let syn::Fields::Named(ref mut fields) = struct_data.fields {
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            /// The bounding rectangle of the element
                            ///
                            /// If this method is not used, then the element will have a size and position of
                            /// zero and will not be visible until its bounding rectangle is set.
                            pub rect: #crate_name::math::Rect
                        })
                        .unwrap(),
                );
            }

            quote! {
                #ast

                impl #impl_generics #name #ty_generics #where_clause {
                    /// The bounding rectangle of the element
                    ///
                    /// If this method is not used, then the element will have a size and position of
                    /// zero and will not be visible until its bounding rectangle is set.
                    pub const fn rect(mut self, rect: #crate_name::math::Rect) -> Self {
                        self.rect = rect;
                        self
                    }
                }
            }
            .into()
        }
        _ => syn::Error::new(
            ast.span(),
            "`element_builder_rect` has to be used with structs ",
        )
        .to_compile_error()
        .into(),
    }
}

#[proc_macro_attribute]
pub fn element_builder_hidden(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident.clone();
    let generics = ast.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            if let syn::Fields::Named(ref mut fields) = struct_data.fields {
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            /// Whether or not this element is manually hidden
                            ///
                            /// By default this is set to `false`.
                            pub manually_hidden: bool
                        })
                        .unwrap(),
                );
            }

            quote! {
                #ast

                impl #impl_generics #name #ty_generics #where_clause {
                    /// Whether or not this element is manually hidden
                    ///
                    /// By default this is set to `false`.
                    pub const fn hidden(mut self, hidden: bool) -> Self {
                        self.manually_hidden = hidden;
                        self
                    }
                }
            }
            .into()
        }
        _ => syn::Error::new(
            ast.span(),
            "`element_builder_rect` has to be used with structs ",
        )
        .to_compile_error()
        .into(),
    }
}

#[proc_macro_attribute]
pub fn element_builder_disabled(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident.clone();
    let generics = ast.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            if let syn::Fields::Named(ref mut fields) = struct_data.fields {
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            /// Whether or not this element is in the disabled state
                            ///
                            /// By default this is set to `false`.
                            pub disabled: bool
                        })
                        .unwrap(),
                );
            }

            quote! {
                #ast

                impl #impl_generics #name #ty_generics #where_clause {
                    /// Whether or not this element is in the disabled state
                    ///
                    /// By default this is set to `false`.
                    pub const fn disabled(mut self, disabled: bool) -> Self {
                        self.disabled = disabled;
                        self
                    }
                }
            }
            .into()
        }
        _ => syn::Error::new(
            ast.span(),
            "`element_builder_disabled` has to be used with structs ",
        )
        .to_compile_error()
        .into(),
    }
}

#[proc_macro_attribute]
pub fn element_builder_tooltip(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident.clone();
    let generics = ast.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let crate_name = root();

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            if let syn::Fields::Named(ref mut fields) = struct_data.fields {
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            /// The tooltip data
                            ///
                            /// If this is `None` then this element will not have a tooltip.
                            ///
                            /// By default this is set to `None`.
                            pub tooltip_data: Option<#crate_name::elements::tooltip::TooltipData>
                        })
                        .unwrap(),
                );
            }

            quote! {
                #ast

                impl #impl_generics #name #ty_generics #where_clause {
                    /// Show a tooltip when the user hovers over this element
                    ///
                    /// * `text` - The tooltip text
                    /// * `align` - Where to align the tooltip relative to this element
                    pub fn tooltip(mut self, text: impl Into<String>, align: #crate_name::layout::Align2) -> Self {
                        self.tooltip_data = Some(#crate_name::elements::tooltip::TooltipData::new(text, align));
                        self
                    }
                }
            }
            .into()
        }
        _ => syn::Error::new(
            ast.span(),
            "`element_builder_tooltip` has to be used with structs ",
        )
        .to_compile_error()
        .into(),
    }
}

#[proc_macro_attribute]
pub fn element_handle(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident.clone();
    let generics = ast.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let crate_name = root();

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            if let syn::Fields::Named(ref mut fields) = struct_data.fields {
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            el: #crate_name::prelude::ElementHandle
                        })
                        .unwrap(),
                );
            }

            quote! {
                #ast

                impl #impl_generics #name #ty_generics #where_clause {
                    /// Get the bounding rectangle of this element instance.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn rect(&self) -> #crate_name::math::Rect {
                        self.el.rect()
                    }

                    /// Get the top-left position of the bounding rectangle of this element instance.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn origin(&self) -> #crate_name::math::Point {
                        self.el.rect().origin
                    }

                    /// Get the bottom-right position of the bounding rectangle of this element instance.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn max(&self) -> #crate_name::math::Point {
                        self.el.rect().max()
                    }

                    /// Get the center position of the bounding rectangle of this element instance.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn center(&self) -> #crate_name::math::Point {
                        self.el.rect().center()
                    }

                    /// Get the x position of the left edge of the bounding rectangle of this element instance.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn min_x(&self) -> f32 {
                        self.el.rect().min_x()
                    }

                    /// Get the y position of the top edge of the bounding rectangle of this element instance.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn min_y(&self) -> f32 {
                        self.el.rect().min_y()
                    }

                    /// Get the x position of the right edge of the bounding rectangle of this element instance.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn max_x(&self) -> f32 {
                        self.el.rect().max_x()
                    }

                    /// Get the y position of the bottom edge of the bounding rectangle of this element instance.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn max_y(&self) -> f32 {
                        self.el.rect().max_y()
                    }

                    /// Get the size of the bounding rectangle of this element instance.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn size(&self) -> Size {
                        self.el.rect().size
                    }

                    /// Get the width of the bounding rectangle of this element instance.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn width(&self) -> f32 {
                        self.el.rect().width()
                    }

                    /// Get the height of the bounding rectangle of this element instance.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn height(&self) -> f32 {
                        self.el.rect().height()
                    }

                    /// Get the z index of this element instance.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn z_index(&self) -> #crate_name::math::ZIndex {
                        self.el.z_index()
                    }

                    /// Returns `true` if the element instance has been manually hidden.
                    ///
                    /// Note that even if this returns `true`, the element may still be hidden
                    /// due to it being outside of the render area.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn manually_hidden(&self) -> bool {
                        self.el.manually_hidden()
                    }

                    /// Set the z index of this element instance.
                    ///
                    /// An update will only be sent to the view if the z index has changed.
                    ///
                    /// Returns `true` if the z index has changed.
                    ///
                    /// This will *NOT* trigger an element update unless the value has changed,
                    /// so this method is very cheap to call frequently.
                    pub fn set_z_index(&mut self, z_index: ZIndex) -> bool {
                        self.el.set_z_index(z_index)
                    }

                    /// Set to hide or show this element instance.
                    ///
                    /// Note, there is no need to hide elements just because they appear outside
                    /// of the render area. The view already handles that for you.
                    ///
                    /// An update will only be sent to the view if the visibility request
                    /// has changed since the previous call.
                    ///
                    /// Returns `true` if the hidden state has changed.
                    ///
                    /// This will *NOT* trigger an element update unless the value has changed,
                    /// so this method is very cheap to call frequently.
                    pub fn set_hidden(&mut self, hidden: bool) -> bool {
                        self.el.set_hidden(hidden)
                    }

                    /// Get the actual bounding rectangle of this element, accounting for the offset
                    /// introduced by its assigned scissoring rectangle.
                    pub fn rect_in_window<A_: Clone + 'static>(&self, cx: &#crate_name::WindowContext<'_, A_>) -> Rect {
                        self.el.rect_in_window(cx)
                    }
                }
            }
            .into()
        }
        _ => syn::Error::new(ast.span(), "`element_handle` has to be used with structs ")
            .to_compile_error()
            .into(),
    }
}

#[proc_macro_attribute]
pub fn element_handle_set_rect(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident.clone();
    let generics = ast.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let crate_name = root();

    match &ast.data {
        syn::Data::Struct(_) => {
            quote! {
                #ast

                impl #impl_generics #name #ty_generics #where_clause {
                    /// Set the rectangular area of this element instance.
                    ///
                    /// An update will only be sent to the view if the rectangle has changed.
                    ///
                    /// Returns `true` if the rectangle has changed.
                    ///
                    /// This will *NOT* trigger an element update unless the value has changed,
                    /// so this method is very cheap to call frequently.
                    pub fn set_rect(&mut self, rect: #crate_name::math::Rect) -> bool {
                        self.el.set_rect(rect)
                    }

                    /// Set the position of the rectangular area of this element instance.
                    ///
                    /// An update will only be sent to the view if the rectangle has changed.
                    ///
                    /// Note, it is more efficient to use `ElementHandle::set_rect()` than
                    /// to set the position and size separately.
                    ///
                    /// Returns `true` if the position has changed.
                    ///
                    /// This will *NOT* trigger an element update unless the value has changed,
                    /// so this method is very cheap to call frequently.
                    pub fn set_pos(&mut self, pos: #crate_name::math::Point) -> bool {
                        self.el.set_pos(pos)
                    }

                    /// Set the size of the rectangular area of this element instance.
                    ///
                    /// An update will only be sent to the view if the rectangle has changed.
                    ///
                    /// Note, it is more efficient to use `ElementHandle::set_rect()` than
                    /// to set the position and size separately.
                    ///
                    /// Returns `true` if the size has changed.
                    ///
                    /// This will *NOT* trigger an element update unless the value has changed,
                    /// so this method is very cheap to call frequently.
                    pub fn set_size(&mut self, size: #crate_name::math::Size) -> bool {
                        self.el.set_size(size)
                    }

                    /// Set the x position of the rectangular area of this element instance.
                    ///
                    /// An update will only be sent to the view if the rectangle has changed.
                    ///
                    /// Note, it is more efficient to use `ElementHandle::set_pos()` or
                    /// `ElementHandle::set_rect()` than to set the fields of the rectangle
                    /// separately.
                    ///
                    /// Returns `true` if the x position has changed.
                    ///
                    /// This will *NOT* trigger an element update unless the value has changed,
                    /// so this method is very cheap to call frequently.
                    pub fn set_x(&mut self, x: f32) -> bool {
                        self.el.set_x(x)
                    }

                    /// Set the y position of the rectangular area of this element instance.
                    ///
                    /// An update will only be sent to the view if the rectangle has changed.
                    ///
                    /// Note, it is more efficient to use `ElementHandle::set_pos()` or
                    /// `ElementHandle::set_rect()` than to set the fields of the rectangle
                    /// separately.
                    ///
                    /// Returns `true` if the y position has changed.
                    ///
                    /// This will *NOT* trigger an element update unless the value has changed,
                    /// so this method is very cheap to call frequently.
                    pub fn set_y(&mut self, y: f32) -> bool {
                        self.el.set_y(y)
                    }

                    /// Set the width of the rectangular area of this element instance.
                    ///
                    /// An update will only be sent to the view if the rectangle has changed.
                    ///
                    /// Note, it is more efficient to use `ElementHandle::set_size()` or
                    /// `ElementHandle::set_rect()` than to set the fields of the rectangle
                    /// separately.
                    ///
                    /// Returns `true` if the width has changed.
                    ///
                    /// This will *NOT* trigger an element update unless the value has changed,
                    /// so this method is very cheap to call frequently.
                    pub fn set_width(&mut self, width: f32) -> bool {
                        self.el.set_width(width)
                    }

                    /// Set the height of the rectangular area of this element instance.
                    ///
                    /// An update will only be sent to the view if the rectangle has changed.
                    ///
                    /// Note, it is more efficient to use `ElementHandle::set_size()` or
                    /// `ElementHandle::set_rect()` than to set the fields of the rectangle
                    /// separately.
                    ///
                    /// Returns `true` if the height has changed.
                    ///
                    /// This will *NOT* trigger an element update unless the value has changed,
                    /// so this method is very cheap to call frequently.
                    pub fn set_height(&mut self, height: f32) -> bool {
                        self.el.set_height(height)
                    }

                    /// Offset the element's rectangular area.
                    ///
                    /// Note, this will *always* cause an element update even if the offset
                    /// is zero, so prefer to call this method sparingly.
                    pub fn offset_pos(&mut self, offset: #crate_name::math::Vector) {
                        self.el.offset_pos(offset)
                    }
                }
            }
            .into()
        }
        _ => syn::Error::new(
            ast.span(),
            "`element_handle_set_rect` has to be used with structs ",
        )
        .to_compile_error()
        .into(),
    }
}

#[proc_macro_attribute]
pub fn element_handle_class(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident.clone();
    let generics = ast.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let crate_name = root();

    match &ast.data {
        syn::Data::Struct(_) => {
            quote! {
                #ast

                impl #impl_generics #name #ty_generics #where_clause {
                    /// The current style class of the element.
                    ///
                    /// This is cached directly in the handle so this is very cheap to call frequently.
                    pub fn class(&self) -> #crate_name::style::ClassID {
                        self.el.class()
                    }

                    /// Set the class of the element.
                    ///
                    /// Returns `true` if the class has changed.
                    ///
                    /// This will *NOT* trigger an element update unless the value has changed,
                    /// and the class ID is cached in the handle itself, so this is relatively
                    /// cheap to call frequently (although a String comparison is performed).
                    pub fn set_class(&mut self, class: #crate_name::style::ClassID) -> bool {
                        self.el.set_class(class)
                    }
                }
            }
            .into()
        }
        _ => syn::Error::new(
            ast.span(),
            "`element_handle_class` has to be used with structs ",
        )
        .to_compile_error()
        .into(),
    }
}

#[proc_macro_attribute]
pub fn element_handle_layout_aligned(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident.clone();
    let generics = ast.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let crate_name = root();

    match &ast.data {
        syn::Data::Struct(_) => {
            quote! {
                #ast

                impl #impl_generics #name #ty_generics #where_clause {
                    /// Layout the element aligned to the given point.
                    ///
                    /// Returns `true` if the layout has changed.
                    ///
                    /// This will *NOT* trigger an element update unless the value has changed,
                    /// so this method is relatively cheap to call frequently.
                    pub fn layout_aligned(&mut self, size: #crate_name::math::Size, point: #crate_name::math::Point, align: #crate_name::layout::Align2) -> bool {
                        self.el.set_rect(align.align_rect_to_point(point, size))
                    }
                }
            }
            .into()
        }
        _ => syn::Error::new(
            ast.span(),
            "`element_handle_layout_aligned` has to be used with structs ",
        )
        .to_compile_error()
        .into(),
    }
}

#[proc_macro_attribute]
pub fn element_handle_set_tooltip(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident.clone();
    let generics = ast.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let crate_name = root();

    match &ast.data {
        syn::Data::Struct(_) => {
            quote! {
                #ast

                impl #impl_generics #name #ty_generics #where_clause {
                    /// Set the tooltip to show when the user hovers over this element
                    ///
                    /// Returns `true` if the tooltip data has changed.
                    ///
                    /// * `text` - The tooltip text. If this is `None` then no tooltip will be shown.
                    /// * `align` - Where to align the tooltip relative to this element
                    ///
                    /// This will *NOT* trigger an element update unless the value has changed,
                    /// so this method is relatively cheap to call frequently (although a string
                    /// comparison is performed).
                    pub fn set_tooltip(mut self, text: Option<&str>, align: #crate_name::layout::Align2) -> bool {
                        if RefCell::borrow_mut(&self.shared_state).tooltip_inner.set_data(text, align) {
                            self.el.notify_custom_state_change();
                            true
                        } else {
                            false
                        }
                    }
                }
            }
            .into()
        }
        _ => syn::Error::new(
            ast.span(),
            "`element_handle_set_tooltip` has to be used with structs ",
        )
        .to_compile_error()
        .into(),
    }
}
