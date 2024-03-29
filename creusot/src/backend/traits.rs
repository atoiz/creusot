use super::{
    clone_map::{CloneLevel, CloneSummary, Dependencies},
    term::lower_pure,
    GraphDepth, Why3Generator,
};
use crate::{
    backend,
    backend::{all_generic_decls_for, own_generic_decls_for, Namer},
    ctx::ItemType,
    util::{self, item_name, module_name},
};
use rustc_hir::{def::Namespace, def_id::DefId};
use rustc_middle::ty::{GenericArgs, GenericArgsRef};
use why3::declaration::{Decl, Goal, Module, TyDecl};

pub(crate) fn lower_impl<'tcx>(ctx: &mut Why3Generator<'tcx>, def_id: DefId) -> Module {
    let tcx = ctx.tcx;
    let data = ctx.trait_impl(def_id).clone();

    let mut names = Dependencies::new(ctx.tcx, [def_id]);

    let mut decls: Vec<_> = own_generic_decls_for(ctx.tcx, def_id).collect();
    let mut refn_decls = Vec::new();

    for refn in &data.refinements {
        let name = item_name(tcx, refn.impl_.0, Namespace::ValueNS);

        decls.extend(own_generic_decls_for(tcx, refn.impl_.0));
        refn_decls.push(Decl::Goal(Goal {
            name: format!("{}_refn", &*name).into(),
            goal: lower_pure(ctx, &mut names, &refn.refn.clone()),
        }));
    }

    let (clones, _) = names.provide_deps(ctx, GraphDepth::Deep);
    decls.extend(clones);
    decls.extend(refn_decls);

    Module { name: module_name(ctx.tcx, def_id), decls }
}

impl<'tcx> Why3Generator<'tcx> {
    pub(crate) fn translate_assoc_ty(&mut self, def_id: DefId) -> (Module, CloneSummary<'tcx>) {
        assert_eq!(util::item_type(self.tcx, def_id), ItemType::AssocTy);

        let mut names = Dependencies::new(self.tcx, [def_id]);

        let mut decls: Vec<_> = all_generic_decls_for(self.tcx, def_id).collect();
        let ty_decl = self.assoc_ty_decl(
            &mut names,
            def_id,
            GenericArgs::identity_for_item(self.tcx, def_id),
        );

        decls.push(ty_decl);

        let (clones, summary) = names.provide_deps(self, GraphDepth::Shallow);
        decls.extend(clones);

        (Module { name: module_name(self.tcx, def_id), decls }, summary)
    }

    // Probably needs to take a pair of id and subst to handle cloning properly
    pub(crate) fn assoc_ty_decl<N: Namer<'tcx>>(
        &mut self,
        names: &mut N,
        def_id: DefId,
        substs: GenericArgsRef<'tcx>,
    ) -> Decl {
        let name = names.ty(def_id, substs).name;

        let ty_decl = match self.tcx.associated_item(def_id).container {
            rustc_middle::ty::ImplContainer => names.with_vis(CloneLevel::Signature, |names| {
                let assoc_ty = self.tcx.type_of(def_id).instantiate_identity();
                TyDecl::Alias {
                    ty_name: name,
                    ty_params: vec![],
                    alias: backend::ty::translate_ty(self, names, rustc_span::DUMMY_SP, assoc_ty),
                }
            }),
            rustc_middle::ty::TraitContainer => TyDecl::Opaque { ty_name: name, ty_params: vec![] },
        };

        Decl::TyDecl(ty_decl)
    }
}
