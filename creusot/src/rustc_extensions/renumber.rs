use rustc_index::vec::IndexVec;
use rustc_infer::infer::{InferCtxt, NllRegionVariableOrigin};
use rustc_middle::{
    mir::{
        visit::{MutVisitor, TyContext},
        Body, Location, Promoted,
    },
    ty::{self, subst::SubstsRef, Ty, TyCtxt, TypeFoldable},
};

/// Replaces all free regions appearing in the MIR with fresh
/// inference variables, returning the number of variables created.
pub(crate) fn renumber_mir<'tcx>(
    infcx: &InferCtxt<'tcx>,
    body: &mut Body<'tcx>,
    promoted: &mut IndexVec<Promoted, Body<'tcx>>,
) {
    let mut visitor = NllVisitor { infcx };

    for body in promoted.iter_mut() {
        visitor.visit_body(body);
    }

    visitor.visit_body(body);
}

/// Replaces all regions appearing in `value` with fresh inference
/// variables.
pub(crate) fn renumber_regions<'tcx, T>(infcx: &InferCtxt<'tcx>, value: T) -> T
where
    T: TypeFoldable<'tcx>,
{
    infcx.tcx.fold_regions(value, |_region, _depth| {
        let origin = NllRegionVariableOrigin::Existential { from_forall: false };
        infcx.next_nll_region_var(origin)
    })
}

struct NllVisitor<'a, 'tcx> {
    infcx: &'a InferCtxt<'tcx>,
}

impl<'a, 'tcx> NllVisitor<'a, 'tcx> {
    fn renumber_regions<T>(&mut self, value: T) -> T
    where
        T: TypeFoldable<'tcx>,
    {
        renumber_regions(self.infcx, value)
    }
}

impl<'a, 'tcx> MutVisitor<'tcx> for NllVisitor<'a, 'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.infcx.tcx
    }

    fn visit_ty(&mut self, ty: &mut Ty<'tcx>, _: TyContext) {
        *ty = self.renumber_regions(*ty);
    }

    fn visit_substs(&mut self, substs: &mut SubstsRef<'tcx>, _: Location) {
        *substs = self.renumber_regions(*substs);
    }

    fn visit_region(&mut self, region: &mut ty::Region<'tcx>, _: Location) {
        let old_region = *region;
        *region = self.renumber_regions(old_region);
    }
}
