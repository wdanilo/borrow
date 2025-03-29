#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::fmt::Debug;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Data, Fields, Type};
use itertools::Itertools;
use proc_macro2::TokenStream;

// =============
// === Utils ===
// =============

fn snake_to_camel(s: &str) -> String {
    s.split('_').map(|s| {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().chain(chars).collect()
        }
    }).collect()
}

fn internal(s: &str) -> String {
    format!("__{s}")
}


// =============
// === Macro ===
// =============

fn meta_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident;
    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields.named.iter().collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let field_types = fields.iter().map(|f| &f.ty).collect_vec();

    let generics_vec = input.generics.params.iter().collect_vec();

    let lifetimes = generics_vec.iter().filter_map(|t| {
        if let syn::GenericParam::Lifetime(lt) = t {
            Some(lt)
        } else {
            None
        }
    }).collect_vec();

    let ty_params = generics_vec.iter().filter_map(|t| {
        if let syn::GenericParam::Type(ty) = t {
            Some(ty.ident.clone())
        } else {
            None
        }
    }).collect_vec();

    let params = quote! {#(#lifetimes,)* #(#ty_params,)*};

    let bounds_vec = {
        let inline_bounds = generics_vec.iter().filter_map(|t| {
            if let syn::GenericParam::Type(ty) = t {
                (!ty.bounds.is_empty()).then_some(ty)
            } else {
                None
            }
        }).collect_vec();

        let where_bounds = input.generics.where_clause.as_ref().map(|t|
            t.predicates.iter().collect_vec()
        ).unwrap_or_default();

        inline_bounds.iter().map(|t| quote! {#t})
            .chain(where_bounds.iter().map(|t| quote! {#t})).collect_vec()

    };

    let bounds = quote! {#(#bounds_vec,)*};

    // === Ctx 1 ===

    let has_fields_for_struct = quote! {
        impl<#params> borrow::HasFields for #ident<#params>
        where #bounds {
            type Fields = borrow::HList![#(#field_types,)*];
        }
    };

    // === Ctx 2 ===

    let has_fields_ext_for_struct = {
        let fields_hidden = field_types.iter().map(|_| quote! {borrow::Hidden});
        let fields_ref    = field_types.iter().map(|t| quote! {&'__a #t});
        let fields_mut    = field_types.iter().map(|t| quote! {&'__a mut #t});
        quote! {
            impl<#params> borrow::HasFieldsExt for #ident<#params>
            where #bounds {
                type FieldsAsHidden = borrow::HList![ #(#fields_hidden,)* ];
                type FieldsAsRef<'__a> = borrow::HList![ #(#fields_ref,)* ] where Self: '__a;
                type FieldsAsMut<'__a> = borrow::HList![ #(#fields_mut,)* ] where Self: '__a;
            }
        }
    };

    // === Ctx 3 ===

    let has_field_for_struct = {
        let impls = fields.iter().enumerate().map(|(i, field)| {
            let field_ident = &field.ident;
            let field_ty = &field.ty;
            let n = Ident::new(&format!("N{i}"), field.ident.as_ref().unwrap().span());
            quote! {
                impl<#params> borrow::HasField<borrow::Str!(#field_ident)>
                for #ident<#params>
                where #bounds {
                    type Type = #field_ty;
                    type Index = borrow::hlist::#n;
                    #[inline(always)]
                    fn take_field(self) -> Self::Type {
                        self.#field_ident
                    }
                }
            }
        }).collect_vec();
        quote! { #(#impls)* }
    };


    let out = quote! {
        #has_fields_for_struct
        #has_fields_ext_for_struct
        #has_field_for_struct
    };

    out.into()
}


fn get_fields(input: &DeriveInput) -> Vec<&syn::Field> {
    if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields.named.iter().collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    }
}

fn get_params(input: &DeriveInput) -> TokenStream {
    let lifetimes = input.generics.params.iter().filter_map(|t| {
        if let syn::GenericParam::Lifetime(lt) = t {
            Some(lt)
        } else {
            None
        }
    }).collect_vec();

    let ty_params = input.generics.params.iter().filter_map(|t| {
        if let syn::GenericParam::Type(ty) = t {
            Some(ty.ident.clone())
        } else {
            None
        }
    }).collect_vec();
    quote! {#(#lifetimes,)* #(#ty_params,)*}
}

fn get_bounds(input: &DeriveInput) -> TokenStream {
    let inline_bounds = input.generics.params.iter().filter_map(|t| {
        if let syn::GenericParam::Type(ty) = t {
            (!ty.bounds.is_empty()).then_some(quote!{#ty})
        } else {
            None
        }
    }).collect_vec();

    let where_bounds = input.generics.where_clause.as_ref().map(|t|
        t.predicates.iter().map(|t| quote!{#t}).collect_vec()
    ).unwrap_or_default();

    quote! {#(#inline_bounds,)* #(#where_bounds,)*}
}

// The internal macro documentation shows expansion parts for the following input:
// ```
// pub struct GeometryCtx {}
// pub struct MaterialCtx {}
// pub struct MeshCtx {}
// pub struct SceneCtx {}
//
// #[derive(borrow::Partial)]
// pub struct Ctx<'t, T: Debug> {
//     pub version: &'t T,
//     pub geometry: GeometryCtx,
//     pub material: MaterialCtx,
//     pub mesh: MeshCtx,
//     pub scene: SceneCtx,
// }
//```
#[allow(clippy::cognitive_complexity)]
#[proc_macro_derive(Partial, attributes(module))]
pub fn partial_borrow_derive(input_raw: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let input_raw2 = input_raw.clone();
    let input = parse_macro_input!(input_raw2 as DeriveInput);

    let ident = &input.ident;
    let fields = get_fields(&input);
    let params = get_params(&input);
    let bounds = get_bounds(&input);

    let fields_vis = fields.iter().map(|f| f.vis.clone()).collect_vec();
    let fields_ident = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect_vec();
    let fields_ty = fields.iter().map(|f| &f.ty).collect_vec();

    // Fields in the form __$upper_case_field__
    let fields_param = fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        Ident::new(&format!("__{}", snake_to_camel(&ident.to_string())), ident.span())
    }).collect_vec();



    let mut out: Vec<TokenStream> = vec![];

    // === Ctx 1 ===

    out.push(meta_derive(input_raw.clone()).into());

    // === CtxRef 1 ===

    let ref_ident = Ident::new(&format!("{ident}Ref"), ident.span());

    // Generates:
    //
    // ```
    // pub struct CtxRef<__S__, __Version, __Geometry, __Material, __Mesh, __Scene> {
    //     pub version: borrow::Field<__Version>,
    //     pub geometry: borrow::Field<__Geometry>,
    //     pub material: borrow::Field<__Material>,
    //     pub mesh: borrow::Field<__Mesh>,
    //     pub scene: borrow::Field<__Scene>,
    //     pub marker: std::marker::PhantomData<__S__>
    // }
    // ```
    let ref_struct_def = {
        quote! {
            pub struct #ref_ident<__S__, #(#fields_param,)*> {
                #(#fields_vis #fields_ident: borrow::Field<#fields_param>,)*
                marker: std::marker::PhantomData<__S__>
            }
        }
    };

    out.push(ref_struct_def.clone());
    out.push(meta_derive(ref_struct_def.into()).into());

    // Generates:
    //
    // ```
    // impl<'t, T, __Version, __Geometry, __Material, __Mesh, __Scene>
    // borrow::AsRefWithFields<borrow::HList![__Version, __Geometry, __Material, __Mesh, __Scene]>
    // for Ctx<'t, T>
    // where T: Debug {
    //     type Output = CtxRef<Ctx<'t, T>, __Version, __Geometry, __Material, __Mesh, __Scene>;
    // }
    // ```
    out.push(
        quote! {
            impl<#params #(#fields_param,)*>
            borrow::AsRefWithFields<borrow::HList![#(#fields_param,)*]>
            for #ident<#params>
            where #bounds {
                type Output = #ref_ident<#ident<#params>, #(#fields_param,)*>;
            }
        }
    );

    // Generates:
    //
    // impl<'__s__, __S__, __Version, __Geometry, __Material, __Mesh, __Scene> borrow::CloneRef<'__s__>
    // for CtxRef<__S__, __Version, __Geometry, __Material, __Mesh, __Scene>
    // where
    //     borrow::Field<__Version>: borrow::CloneField<'__s__>,
    //     borrow::Field<__Geometry>: borrow::CloneField<'__s__>,
    //     borrow::Field<__Material>: borrow::CloneField<'__s__>,
    //     borrow::Field<__Mesh>: borrow::CloneField<'__s__>,
    //     borrow::Field<__Scene>: borrow::CloneField<'__s__>,
    // {
    //     type Cloned = CtxRef<
    //         __S__,
    //         borrow::ClonedField<'__s__, borrow::Field<__Version>>,
    //         borrow::ClonedField<'__s__, borrow::Field<__Geometry>>,
    //         borrow::ClonedField<'__s__, borrow::Field<__Material>>,
    //         borrow::ClonedField<'__s__, borrow::Field<__Mesh>>,
    //         borrow::ClonedField<'__s__, borrow::Field<__Scene>>
    //     >;
    //     fn clone_ref_disabled_usage_tracking(&'__s__ mut self) -> Self::Cloned {
    //         use borrow::CloneField;
    //         CtxRef {
    //             version: self.version.clone_field_disabled_usage_tracking(),
    //             geometry: self.geometry.clone_field_disabled_usage_tracking(),
    //             material: self.material.clone_field_disabled_usage_tracking(),
    //             mesh: self.mesh.clone_field_disabled_usage_tracking(),
    //             scene: self.scene.clone_field_disabled_usage_tracking(),
    //             marker: std::marker::PhantomData,
    //         }
    //     }
    // }
    out.push(
        quote! {
            impl<'__s__, __S__, #(#fields_param,)*> borrow::CloneRef<'__s__>
            for #ref_ident<__S__, #(#fields_param,)*>
            where
                #(borrow::Field<#fields_param>: borrow::CloneField<'__s__>,)*
            {
                type Cloned = #ref_ident<
                    __S__,
                    #(borrow::ClonedField<'__s__, borrow::Field<#fields_param>>,)*
                >;
                fn clone_ref_disabled_usage_tracking(&'__s__ mut self) -> Self::Cloned {
                    use borrow::CloneField;
                    #ref_ident {
                        #(#fields_ident: self.#fields_ident.clone_field_disabled_usage_tracking(),)*
                        marker: std::marker::PhantomData,
                    }
                }
            }
        }
    );

    // Generates:
    //
    // ```
    // #[allow(non_camel_case_types)]
    // #[allow(non_snake_case)]
    // impl<'__a__, __S__,
    //     __Version, __Geometry, __Material, __Mesh, __Scene,
    //     __Version__Target, __Geometry__Target, __Material__Target, __Mesh__Target, __Scene__Target,
    //     __Version__Rest, __Geometry__Rest, __Material__Rest, __Mesh__Rest, __Scene__Rest>
    // borrow::Partial<'__a__, ExplicitParams<__S__, CtxRef<__S__, __Version__Target, __Geometry__Target, __Material__Target, __Mesh__Target, __Scene__Target>>>
    // for CtxRef<__S__, __Version, __Geometry, __Material, __Mesh, __Scene>
    // where
    //     borrow::AcquireMarker: borrow::Acquire<'__a__, __Version, __Version__Target, Rest=__Version__Rest>,
    //     borrow::AcquireMarker: borrow::Acquire<'__a__, __Geometry, __Geometry__Target, Rest=__Geometry__Rest>,
    //     borrow::AcquireMarker: borrow::Acquire<'__a__, __Material, __Material__Target, Rest=__Material__Rest>,
    //     borrow::AcquireMarker: borrow::Acquire<'__a__, __Mesh, __Mesh__Target, Rest=__Mesh__Rest>,
    //     borrow::AcquireMarker: borrow::Acquire<'__a__, __Scene, __Scene__Target, Rest=__Scene__Rest>,
    // {
    //     type Rest = ExplicitParams<__S__, CtxRef<__S__, __Version__Rest, __Geometry__Rest, __Material__Rest, __Mesh__Rest, __Scene__Rest>>;
    //     #[track_caller]
    //     #[inline(always)]
    //     fn split_impl(
    //         &'__a__ mut self
    //     ) -> (
    //         ExplicitParams<
    //             __S__,
    //             CtxRef<
    //                 __S__,
    //                 __Version__Target,
    //                 __Geometry__Target,
    //                 __Material__Target,
    //                 __Mesh__Target,
    //                 __Scene__Target
    //             >
    //         >,
    //         Self::Rest
    //     ) {
    //         use borrow::Acquire;
    //         let (version, __version__rest) = borrow::AcquireMarker::acquire(&mut self.version);
    //         let (geometry, __geometry__rest) = borrow::AcquireMarker::acquire(&mut self.geometry);
    //         let (material, __material__rest) = borrow::AcquireMarker::acquire(&mut self.material);
    //         let (mesh, __mesh__rest) = borrow::AcquireMarker::acquire(&mut self.mesh);
    //         let (scene, __scene__rest) = borrow::AcquireMarker::acquire(&mut self.scene);
    //         (
    //             ExplicitParams::new(
    //                 CtxRef {
    //                     version,
    //                     geometry,
    //                     material,
    //                     mesh,
    //                     scene,
    //                     marker: std::marker::PhantomData
    //                 }
    //             ),
    //             ExplicitParams::new(
    //                 CtxRef {
    //                     version: __version__rest,
    //                     geometry: __geometry__rest,
    //                     material: __material__rest,
    //                     mesh: __mesh__rest,
    //                     scene: __scene__rest,
    //                     marker: std::marker::PhantomData
    //                 }
    //             )
    //         )
    //     }
    // }
    // ```

    out.push({
        let field_params_target = fields_param.iter().map(|i| {
            Ident::new(&format!("{i}{}", internal("Target")), i.span())
        }).collect_vec();

        let field_params_rest = fields_param.iter().map(|i| {
            Ident::new(&format!("{i}{}", internal("Rest")), i.span())
        }).collect_vec();

        let fields_rest_ident = fields_ident.iter().map(|i|
            Ident::new(&format!("{}{}", internal(&i.to_string()), internal("rest")), i.span())
        ).collect_vec();

        quote! {
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            impl<__S__,
                #(#fields_param,)*
                #(#field_params_target,)*
                #(#field_params_rest,)*
            >
            borrow::IntoPartial<borrow::ExplicitParams<__S__, #ref_ident<__S__, #(#field_params_target,)*>>>
            for #ref_ident<__S__, #(#fields_param,)*>
            where
                #(
                    borrow::AcquireMarker: borrow::Acquire<
                        #fields_param,
                        #field_params_target,
                        Rest=#field_params_rest
                    >,
                )*
            {
                type Rest = borrow::ExplicitParams<__S__, #ref_ident<__S__, #(#field_params_rest,)*>>;

                #[track_caller]
                #[inline(always)]
                fn into_split_impl(
                    mut self
                ) -> (
                    borrow::ExplicitParams<__S__, #ref_ident<__S__, #(#field_params_target,)*>>,
                    Self::Rest
                ) {
                    use borrow::Acquire;
                    #(let (#fields_ident, #fields_rest_ident) =
                        borrow::AcquireMarker::acquire(self.#fields_ident);)*
                    (
                        borrow::ExplicitParams::new(
                            #ref_ident {
                                #(#fields_ident,)*
                                marker: std::marker::PhantomData
                            }
                        ),
                        borrow::ExplicitParams::new(
                            #ref_ident {
                                #(#fields_ident: #fields_rest_ident,)*
                                marker: std::marker::PhantomData
                            }
                        )
                    )
                }
            }
        }
    });

    // For each field. For the 'version' field:
    //
    // ```
    // #[allow(non_camel_case_types)]
    // impl<'a, '__s, '__src, '__tgt, 't, T, __Geometry, __Material, __Mesh, __Scene>
    // CtxRef<Ctx<'t, T>, &'__src mut &'t T, __Geometry, __Material, __Mesh, __Scene>
    // where
    //     T: Debug,
    //     (&'t T): '__tgt,
    //     Self: borrow::Partial<
    //         '__s,
    //         CtxRef<
    //             Ctx<'t, T>,
    //             &'__tgt mut &'t T,
    //             borrow::Hidden,
    //             borrow::Hidden,
    //             borrow::Hidden,
    //             borrow::Hidden
    //         >
    //     >
    // {
    //     #[track_caller]
    //     #[inline(always)]
    //     pub fn extract_version(&'__s mut self) -> (
    //         borrow::Field<&'__tgt mut &'t T>,
    //         borrow::ExplicitParams<
    //             Ctx<'t, T>,
    //             <Self as borrow::Partial<
    //                 '__s,
    //                 CtxRef<
    //                     Ctx<'t, T>,
    //                     &'__tgt mut &'t T,
    //                     borrow::Hidden,
    //                     borrow::Hidden,
    //                     borrow::Hidden,
    //                     borrow::Hidden
    //                 >
    //             >>::Rest>
    //     ) {
    //         let split = borrow::SplitHelper::split(self);
    //         (split.0.version, borrow::ExplicitParams::new(split.1))
    //     }
    // }
    // ```
    //
    // For the 'geometry' field:
    //
    // ```
    // impl<'a, '__s, '__src, '__tgt, 't, T, __Version, __Material, __Mesh, __Scene>
    // CtxRef<Ctx<'t, T>, __Version, &'__src mut GeometryCtx, __Material, __Mesh, __Scene>
    // where
    //     T: Debug,
    //     (GeometryCtx): '__tgt,
    //     Self: borrow::CloneRef<'__s>,
    //     borrow::ClonedRef<'__s, Self>: borrow::IntoPartial<
    //         borrow::ExplicitParams<
    //             Ctx<'t, T>,
    //             CtxRef<
    //                 Ctx<'t, T>,
    //                 borrow::Hidden,
    //                 &'__tgt mut GeometryCtx,
    //                 borrow::Hidden,
    //                 borrow::Hidden,
    //                 borrow::Hidden
    //             >
    //         >
    //     >
    // {
    //     #[track_caller]
    //     #[inline(always)]
    //     pub fn extract_geometry(&'__s mut self) -> (
    //         borrow::Field<&'__tgt mut GeometryCtx>,
    //
    //             <borrow::ClonedRef<'__s, Self> as borrow::IntoPartial<
    //                 borrow::ExplicitParams<
    //                     Ctx<'t, T>,
    //                     CtxRef<
    //                         Ctx<'t, T>,
    //                         borrow::Hidden,
    //                         &'__tgt mut GeometryCtx,
    //                         borrow::Hidden,
    //                         borrow::Hidden,
    //                         borrow::Hidden
    //                     >
    //                 >
    //             >>::Rest
    //     ) {
    //         let split = borrow::IntoPartial::into_split_impl(
    //             borrow::CloneRef::clone_ref_disabled_usage_tracking(self)
    //         );
    //         (split.0.value.#field_ident, split.1)
    //     }
    // }
    // ```
    out.extend((0..fields_param.len()).map(|i| {
        let field_ident = &fields_ident[i];
        let field_ty = &fields_ty[i];
        let field_ref_mut = quote! {&'__tgt mut #field_ty};
        let field_ref_mut2 = quote! {&'__src mut #field_ty};

        let mut params2 = fields_param.clone();
        params2.remove(i);

        let mut ref_params = fields_param.iter().map(|t| quote! {#t}).collect_vec();
        ref_params[i] = field_ref_mut2;

        let mut target_params = fields_param.iter().map(|_| quote! {borrow::Hidden}).collect_vec();
        target_params[i] = field_ref_mut.clone();

        let fn_ident = Ident::new(&format!("extract_{field_ident}"), field_ident.span());

        quote! {
            #[allow(non_camel_case_types)]
            impl<'__s, '__src, '__tgt, #params #(#params2,)*>
            #ref_ident<#ident<#params>, #(#ref_params,)*>
            where
                #bounds
                #field_ty: '__tgt,
                Self: borrow::CloneRef<'__s>,
                borrow::ClonedRef<'__s, Self>: borrow::IntoPartial<
                    borrow::ExplicitParams<
                        #ident<#params>,
                        #ref_ident<
                            #ident<#params>,
                            #(#target_params,)*
                        >
                    >
                >
            {
                #[track_caller]
                #[inline(always)]
                pub fn #fn_ident(&'__s mut self) -> (
                    borrow::Field<#field_ref_mut>,
                        <borrow::ClonedRef<'__s, Self> as borrow::IntoPartial<
                            borrow::ExplicitParams<
                                #ident<#params>,
                                #ref_ident<
                                    #ident<#params>,
                                    #(#target_params,)*
                                >
                            >
                        >>::Rest
                ) {
                    let split = borrow::IntoPartial::into_split_impl(
                        borrow::CloneRef::clone_ref_disabled_usage_tracking(self)
                    );
                    (split.0.value.#field_ident, split.1)
                }
            }
        }
    }));


    // Generates:
    //
    // ```
    // impl<__S__, __Version, __Geometry, __Material, __Mesh, __Scene> borrow::HasUsageTrackedFields
    // for CtxRef<__S__, __Version, __Geometry, __Material, __Mesh, __Scene> {
    //     #[inline(always)]
    //     fn disable_field_usage_tracking(&self) {
    //         self.version.disable_usage_tracking();
    //         self.geometry.disable_usage_tracking();
    //         self.material.disable_usage_tracking();
    //         self.mesh.disable_usage_tracking();
    //         self.scene.disable_usage_tracking();
    //     }
    // }
    // ```
    out.push(quote! {
        impl<__S__, #(#fields_param,)*> borrow::HasUsageTrackedFields
        for #ref_ident<__S__, #(#fields_param,)*> {
            #[inline(always)]
            fn disable_field_usage_tracking(&self) {
                #(self.#fields_ident.disable_usage_tracking();)*
            }
        }
    });

    // Generates:
    //
    // ```
    // impl<'t, T> borrow::AsRefsMut for Ctx<'t, T>
    // where T: Debug {
    //     type Target<'__s> = borrow::ExplicitParams<
    //         Self,
    //         borrow::RefWithFields<Ctx<'t, T>, borrow::FieldsAsMut<'__s, Ctx<'t, T>>>
    //     > where Self: '__s;
    //     #[track_caller]
    //     #[inline(always)]
    //     pub fn as_refs_mut<'__s>(&'__s mut self) -> Self::Target<'__s> {
    //         let struct_ref = CtxRef {
    //             version:  borrow::Field::new("version", &mut self.version),
    //             geometry: borrow::Field::new("geometry", &mut self.geometry),
    //             material: borrow::Field::new("material", &mut self.material),
    //             mesh:     borrow::Field::new("mesh", &mut self.mesh),
    //             scene:    borrow::Field::new("scene", &mut self.scene),
    //             marker:   std::marker::PhantomData,
    //         };
    //         borrow::HasUsageTrackedFields::disable_field_usage_tracking(&struct_ref);
    //         borrow::ExplicitParams::new(struct_ref)
    //     }
    // }
    // ```
    out.push(quote! {
        impl<#params> borrow::AsRefsMut for #ident<#params>
        where #bounds {
            type Target<'__s> = borrow::ExplicitParams<
                Self,
                borrow::RefWithFields<#ident<#params>, borrow::FieldsAsMut<'__s, #ident<#params>>>
            > where Self: '__s;
            #[track_caller]
            #[inline(always)]
            fn as_refs_mut<'__s>(&'__s mut self) -> Self::Target<'__s> {
                let struct_ref = #ref_ident {
                    #(
                        #fields_ident: borrow::Field::new(
                            stringify!(#fields_ident),
                            &mut self.#fields_ident
                        ),
                    )*
                    marker: std::marker::PhantomData,
                };
                borrow::HasUsageTrackedFields::disable_field_usage_tracking(&struct_ref);
                borrow::ExplicitParams::new(struct_ref)
            }
        }
    });

    let output = quote! {
        #(#out)*
    };

    // println!("OUTPUT:\n{}", output);
    output.into()
}

use syn::Token;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::token::Token;

#[derive(Debug)]
enum Selector {
    Ident { lifetime: Option<TokenStream>, is_mut: bool, ident: Ident },
    Star { lifetime: Option<TokenStream>, is_mut: bool }
}

enum Selectors {
    List(Vec<Selector>),
    All
}

impl Selectors {
    fn is_all(&self) -> bool {
        matches!(self, Selectors::All)
    }
}

// #[derive(Debug)]
struct MyInput {
    lifetime: Option<TokenStream>,
    selectors: Selectors,
    target: Type,
}

fn parse_angled_list<T: Parse>(input: ParseStream) -> syn::Result<Vec<T>> {
    let mut params = vec![];

    while !input.peek(Token![>]) {
        if let Ok(value) = input.parse::<T>() {
            params.push(value);
        } else {
            break
        }

        if input.peek(Token![>]) {
            break;
        }

        input.parse::<Token![,]>().ok();
    }

    Ok(params)
}


impl Parse for Selector {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lifetime = input.parse::<syn::Lifetime>().ok().map(|t| quote! { #t });
        let is_mut = input.parse::<Token![mut]>().is_ok();
        if input.parse::<Token![*]>().is_ok() {
            Ok(Selector::Star{ lifetime, is_mut })
        } else {
            let ident: Ident = input.parse()?;
            Ok(Selector::Ident{ lifetime, is_mut, ident })
        }
    }
}

impl Parse for MyInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![&]>()?;

        let lifetime = input.parse::<syn::Lifetime>().ok().map(|t| quote! { #t });

        let selectors = if input.parse::<Token![mut]>().is_ok() {
            Selectors::All
        } else if input.parse::<Token![<]>().is_ok() {
            let selectors = parse_angled_list::<Selector>(input)?;
            input.parse::<Token![>]>()?;
            Selectors::List(selectors)
        } else {
            Selectors::List(vec![])
        };

        let target: Type = input.parse()?;

        Ok(MyInput {
            lifetime,
            selectors,
            target,
        })
    }
}

#[allow(clippy::cognitive_complexity)]
#[proc_macro]
pub fn partial(input_raw: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input_raw as MyInput);

    let target_ident = match &input.target {
        Type::Path(type_path) if type_path.path.segments.len() == 1 && input.selectors.is_all() => {
            let ident = &type_path.path.segments[0].ident;
            let is_lower = ident.to_string().chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
            is_lower.then_some(&type_path.path.segments[0].ident)
        }
        _ => None,
    };

    let out = if let Some(target_ident) = target_ident {
        quote! {
            #target_ident.partial_borrow()
        }
    } else {
        let target = &input.target;
        let default_lifetime = input.lifetime.unwrap_or_else(|| quote!{ '_ });
        let mut out = quote! { borrow::FieldsAsHidden<#target> };
        match &input.selectors {
            Selectors::All => out = quote! {
                borrow::FieldsAsMut <#default_lifetime, #target>
            },
            Selectors::List(selectors) => {
                for selector in selectors {
                    out = match selector {
                        Selector::Ident { lifetime, is_mut, ident } => {
                            let lt = lifetime.as_ref().unwrap_or(&default_lifetime);
                            if *is_mut {
                                quote! { borrow::SetFieldAsMut <#lt, #target, borrow::Str!(#ident), #out>   }
                            } else {
                                quote! { borrow::SetFieldAsRef <#lt, #target, borrow::Str!(#ident), #out>   }
                            }
                        }
                        Selector::Star { lifetime, is_mut } => {
                            let lt = lifetime.as_ref().unwrap_or(&default_lifetime);
                            if *is_mut {
                                quote! { borrow::FieldsAsMut <#lt, #target>   }
                            } else {
                                quote! { borrow::FieldsAsRef <#lt, #target>   }
                            }
                        }
                    }
                }
            }
        }


        out = quote! {
            borrow::ExplicitParams<
                #target,
                borrow::RefWithFields< #target, #out >
            >
        };
        out
    };


    // println!("{}", out);

    // let out = quote! {
    //
    // };
    out.into()
}
