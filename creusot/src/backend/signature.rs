use rustc_hir::{def::Namespace, def_id::DefId};
use why3::{
    declaration::{Contract, Signature},
    exp::{Binder, Trigger},
};

use crate::{
    backend,
    translation::specification::PreContract,
    util::{
        ident_of, item_name, should_replace_trigger, why3_attrs, AnonymousParamName, PreSignature,
    },
};

use super::{logic::function_call, term::lower_pure, CloneLevel, Namer, Why3Generator};

pub(crate) fn signature_of<'tcx, N: Namer<'tcx>>(
    ctx: &mut Why3Generator<'tcx>,
    names: &mut N,
    def_id: DefId,
) -> Signature {
    debug!("signature_of {def_id:?}");
    let pre_sig = ctx.sig(def_id).clone();
    sig_to_why3(ctx, names, &pre_sig, def_id)
}

pub(crate) fn sig_to_why3<'tcx, N: Namer<'tcx>>(
    ctx: &mut Why3Generator<'tcx>,
    names: &mut N,
    pre_sig: &PreSignature<'tcx>,
    // FIXME: Get rid of this def id
    // The PreSig should have the name and the id should be replaced by a param env (if by anything at all...)
    def_id: DefId,
) -> Signature {
    let contract = names
        .with_vis(CloneLevel::Contract, |names| contract_to_why3(&pre_sig.contract, ctx, names));

    let name = item_name(ctx.tcx, def_id, Namespace::ValueNS);

    let span = ctx.tcx.def_span(def_id);
    let args: Vec<Binder> = names.with_vis(CloneLevel::Signature, |names| {
        pre_sig
            .inputs
            .iter()
            .enumerate()
            .map(|(ix, (id, _, ty))| {
                let ty = backend::ty::translate_ty(ctx, names, span, *ty);
                let id = if id.is_empty() {
                    format!("{}", AnonymousParamName(ix)).into()
                } else {
                    ident_of(*id)
                };
                Binder::typed(id, ty)
            })
            .collect()
    });

    let mut attrs = why3_attrs(ctx.tcx, def_id);

    def_id
        .as_local()
        .map(|d| ctx.def_span(d))
        .and_then(|span| ctx.span_attr(span))
        .map(|attr| attrs.push(attr));

    let retty = names.with_vis(CloneLevel::Signature, |names| {
        backend::ty::translate_ty(ctx, names, span, pre_sig.output)
    });

    let mut sig = Signature { name, trigger: None, attrs, retty: Some(retty), args, contract };
    let trigger = if ctx.opts.simple_triggers && should_replace_trigger(ctx.tcx, def_id) {
        Some(Trigger::single(function_call(&sig)))
    } else {
        None
    };
    sig.trigger = trigger;
    sig
}

fn contract_to_why3<'tcx, N: Namer<'tcx>>(
    pre: &PreContract<'tcx>,
    ctx: &mut Why3Generator<'tcx>,
    names: &mut N,
) -> Contract {
    let mut out = Contract::new();
    for term in &pre.requires {
        out.requires.push(lower_pure(ctx, names, term));
    }
    for term in &pre.ensures {
        out.ensures.push(lower_pure(ctx, names, &term));
    }

    if let Some(term) = &pre.variant {
        out.variant = vec![lower_pure(ctx, names, &term)];
    }

    out
}
