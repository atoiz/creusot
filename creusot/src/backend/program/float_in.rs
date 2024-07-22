use std::collections::HashSet;
use why3::{
    coma::{self, Arg, Param, Term, Var},
    ty::Type,
    Ident, QName,
};

enum ExprWFVKind {
    Lambda(Vec<Param>, Box<ExprWFV>),

    Defn(Box<ExprWFV>, bool, Vec<DefnWFV>),

    Let(Box<ExprWFV>, Vec<Var>),

    Symbol(QName),

    App(Box<ExprWFV>, Box<ArgWFV>),

    Assign(Box<ExprWFV>, Vec<(Ident, Term)>),

    Assert(Box<Term>, Box<ExprWFV>),

    Assume(Box<Term>, Box<ExprWFV>),

    BlackBox(Box<ExprWFV>),

    WhiteBox(Box<ExprWFV>),

    Any,
}

struct ExprWFV {
    fvs: HashSet<QName>,
    kind: ExprWFVKind,
}

struct ArgWFV {
    fvs: HashSet<QName>,
    kind: ArgKind,
}

enum ArgKind {
    Ty(Type),
    Term(Term),
    Ref(Ident),
    Cont(ExprWFV),
}

struct DefnWFV {
    name: Ident,
    writes: Vec<Ident>,
    params: Vec<Param>,
    body: ExprWFV,
    fvs: HashSet<QName>,
}

pub(crate) fn float_bindings(expr: coma::Expr) -> coma::Expr {
    let annot = annot(expr);
    float_in(Vec::new(), annot)
}

impl ExprWFV {
    fn unfold(self) -> (ExprWFV, Vec<ArgWFV>) {
        match self.kind {
            ExprWFVKind::App(f, a) => {
                let (f, mut args) = f.unfold();
                args.push(*a);
                (f, args)
            }
            _ => (self, vec![]),
        }
    }
}

fn annot_arg(arg: coma::Arg) -> ArgWFV {
    match arg {
        Arg::Ty(ty) => ArgWFV { fvs: Default::default(), kind: ArgKind::Ty(ty) },
        Arg::Term(t) => {
            ArgWFV { fvs: t.fvs().into_iter().map(|i| i.into()).collect(), kind: ArgKind::Term(t) }
        }
        Arg::Ref(i) => {
            ArgWFV { fvs: vec![i.clone().into()].into_iter().collect(), kind: ArgKind::Ref(i) }
        }
        Arg::Cont(e) => {
            let e = annot(e);
            ArgWFV { fvs: e.fvs.clone(), kind: ArgKind::Cont(e) }
        }
    }
}
fn annot_def(def: coma::Defn) -> DefnWFV {
    let e = annot(def.body);

    let mut fvs = e.fvs.clone();

    // if fvs.contains(&QName::from_string("old_2_0").unwrap()) {
    //     // for d in &defs {
    //         // eprintln!("{:?}", d.fvs);
    //     // }
    //     use why3::Print;
    //     eprintln!("def has var {}", def.name.display());
    // }
    fvs.remove(&def.name.clone().into());
    def.params.iter().for_each(|p| match p {
        Param::Ty(_) => (),
        Param::Term(i, _) => {
            fvs.remove(&i.clone().into());
        }
        Param::Reference(i, _) => {
            fvs.remove(&i.clone().into());
        }
        Param::Cont(_, _, _) => (),
        // Param::Cont(i, _, _) => {
        //     fvs.remove(&i.clone().into());
        // }
    });

    DefnWFV { name: def.name, writes: def.writes, params: def.params, body: e, fvs }
}

/// Takes a coma expression and returns an equivalent expression annotated with its free variables at each level
///
/// TODO: Improve the FV representation to take advantage of sharing.
fn annot(expr: coma::Expr) -> ExprWFV {
    match expr {
        coma::Expr::Symbol(v) => {
            ExprWFV { fvs: [v.clone()].into_iter().collect(), kind: ExprWFVKind::Symbol(v) }
        }
        coma::Expr::App(l, r) => {
            let l = annot(*l);
            let r = annot_arg(*r);
            let mut fvs = l.fvs.clone();
            fvs.extend(r.fvs.clone());

            ExprWFV { fvs, kind: ExprWFVKind::App(Box::new(l), Box::new(r)) }
        }
        coma::Expr::Lambda(bndrs, e) => {
            let e = annot(*e);

            let mut fvs = e.fvs.clone();

            bndrs.iter().for_each(|p| match p {
                Param::Ty(_) => (),
                Param::Term(i, _) => {
                    fvs.remove(&i.clone().into());
                }
                Param::Reference(i, _) => {
                    fvs.remove(&i.clone().into());
                }
                Param::Cont(_, _, _) => (),
                // Param::Cont(i, _, _) => {
                //     fvs.remove(&i.clone().into());
                // }
            });

            ExprWFV { fvs, kind: ExprWFVKind::Lambda(bndrs, Box::new(e)) }
        }
        coma::Expr::Defn(e, b, defs) => {
            let e = annot(*e);
            let defs: Vec<_> = defs.into_iter().map(annot_def).collect();

            let mut fvs: HashSet<_> = e.fvs.clone();

            defs.iter().for_each(|def| fvs.extend(def.fvs.clone()));

            ExprWFV { fvs, kind: ExprWFVKind::Defn(Box::new(e), b, defs) }
        }
        coma::Expr::Assign(e, ts) => {
            let e = annot(*e);
            let mut fvs = e.fvs.clone();
            ts.iter().for_each(|(id, exp)| {
                fvs.insert(id.clone().into());
                fvs.extend(exp.fvs().into_iter().map(|i| i.into()))
            });

            ExprWFV { fvs, kind: ExprWFVKind::Assign(Box::new(e), ts) }
        }
        coma::Expr::Let(e, vars) => {
            let e = annot(*e);

            let mut fvs = e.fvs.clone();
            vars.iter().for_each(|Var(id, _, e, _)| {
                fvs.insert(id.clone().into());
                fvs.extend(e.fvs().into_iter().map(|i| i.into()))
            });

            ExprWFV { fvs, kind: ExprWFVKind::Let(Box::new(e), vars) }
        }
        coma::Expr::Assert(t, e) => {
            let e = annot(*e);
            let fvs = t.fvs().into_iter().map(|i| i.into()).chain(e.fvs.clone()).collect();
            // use why3::Print;
            // eprintln!("fvs: {:?} {}", fvs, t.display());
            ExprWFV { fvs, kind: ExprWFVKind::Assert(t, Box::new(e)) }
        }
        coma::Expr::Assume(t, e) => {
            let e = annot(*e);
            let fvs = t.fvs().into_iter().map(|i| i.into()).chain(e.fvs.clone()).collect();

            ExprWFV { fvs, kind: ExprWFVKind::Assume(t, Box::new(e)) }
        }
        coma::Expr::BlackBox(e) => {
            let e = annot(*e);
            ExprWFV { fvs: e.fvs.clone(), kind: ExprWFVKind::BlackBox(Box::new(e)) }
        }
        coma::Expr::WhiteBox(e) => {
            let e = annot(*e);
            ExprWFV { fvs: e.fvs.clone(), kind: ExprWFVKind::WhiteBox(Box::new(e)) }
        }
        coma::Expr::Any => ExprWFV { fvs: Default::default(), kind: ExprWFVKind::Any },
    }
}

/// Attempts to push the `lets` as far down into `expr` as possible, returns the resulting expression
/// This function will *never* duplicate lets, though it can drop them if they are unneeded.
fn float_in(mut lets: Vec<Var>, mut expr: ExprWFV) -> coma::Expr {
    use coma::*;
    match expr.kind {
        ExprWFVKind::Lambda(pars, body) => {
            let bound = body.fvs.difference(&expr.fvs).cloned().collect();
            // Drop any lets that are shadowed
            // In theory this should not happen
            let (_, inner) = split_floats(lets, &bound);
            // TODO: Account for bound variables
            Expr::Lambda(pars, Box::new(float_in(inner, *body)))
            // wrap_floats(lets, fvs, |inner| Expr::Lambda(pars, Box::new(float_in(inner, *body))))
        }
        ExprWFVKind::Defn(e, b, defs) => {
            let (common, mut def_lets) =
                split_for_case(lets, defs.iter().map(|d| d.fvs.clone()).collect());
            let e = float_in(Vec::new(), *e);

            // if expr.fvs.contains(&QName::from_string("old_2_0").unwrap()) {
            //     for d in &defs {
            //         eprintln!("{:?}", d.fvs);
            //     }
            // }

            let mut defs: Vec<_> =
                defs.into_iter().rev().map(|d| float_in_defn(def_lets.pop().unwrap(), d)).collect();

            defs.reverse();
            Expr::Defn(Box::new(e), b, defs).with_(common)
        }
        ExprWFVKind::Let(e, l) => {
            lets.extend(l);
            let e = float_in(lets, *e);
            e
        }
        ExprWFVKind::Symbol(s) => Expr::Symbol(s).with_(lets),
        ExprWFVKind::App(_, _) => {
            let (f, args) = expr.unfold();
            let (common, mut each) = split_for_case(
                lets,
                std::iter::once(f.fvs.clone()).chain(args.iter().map(|a| a.fvs.clone())).collect(),
            );

            let f = float_in(each.remove(0), f);
            let mut unfloated = Vec::new();
            let args = args
                .into_iter()
                .map(|r| {
                    let mut arg_floats = each.remove(0);
                    let arg = match r.kind {
                        ArgKind::Ty(t) => Arg::Ty(t),
                        ArgKind::Term(term) => Arg::Term(term),
                        ArgKind::Ref(i) => Arg::Ref(i),
                        ArgKind::Cont(c) => {
                            let c = float_in(arg_floats, c);
                            arg_floats = Vec::new();
                            Arg::Cont(c)
                        }
                    };
                    unfloated.extend(arg_floats);
                    arg
                })
                .collect();
            let app = f.app(args);

            let app = app.with_(unfloated);
            app.with_(common)
        }
        ExprWFVKind::Assign(e, asgns) => {
            let fvs = expr.fvs.difference(&e.fvs).cloned().collect();
            wrap_floats(lets, fvs, |inner| Expr::Assign(Box::new(float_in(inner, *e)), asgns))
        }
        ExprWFVKind::Assert(t, e) => {
            let fvs = expr.fvs.difference(&e.fvs).cloned().collect();
            wrap_floats(lets, fvs, |inner| Expr::Assert(t, Box::new(float_in(inner, *e))))
        }
        ExprWFVKind::Assume(t, e) => {
            let fvs = expr.fvs.difference(&e.fvs).cloned().collect();
            wrap_floats(lets, fvs, |inner| Expr::Assume(t, Box::new(float_in(inner, *e))))
        }
        ExprWFVKind::BlackBox(e) => Expr::BlackBox(Box::new(float_in(lets, *e))),
        ExprWFVKind::WhiteBox(e) => Expr::WhiteBox(Box::new(float_in(lets, *e))),
        ExprWFVKind::Any => Expr::Any,
    }
}

fn float_in_defn(lets: Vec<Var>, defn: DefnWFV) -> coma::Defn {
    let DefnWFV { name, writes, params, body, fvs } = defn;
    let fvs = fvs.difference(&body.fvs).cloned().collect();
    let body = wrap_floats(lets, fvs, |inner| float_in(inner, body));
    coma::Defn { name, writes, params, body }
}

fn split_for_case(
    lets: Vec<Var>,
    // _: HashSet<QName>,
    defs: Vec<HashSet<QName>>,
) -> (Vec<Var>, Vec<Vec<Var>>) {
    let mut common = Vec::new();
    let mut branches = vec![Vec::new(); defs.len()];
    for v in lets {
        let occurs: Vec<_> = defs
            .iter()
            .enumerate()
            .filter(|(_, d)| d.contains(&v.0.clone().into()))
            .map(|(ix, _)| ix)
            .collect();

        let here = occurs.len() == 1;
        if here {
            branches[occurs[0]].push(v)
        } else {
            common.push(v);
        }
    }
    (common, branches)
}

/// Wraps `lets` around the given expression, pushing as many bindings as possible to the inner expression.
fn wrap_floats(
    lets: Vec<Var>,
    fvs: HashSet<QName>,
    f: impl FnOnce(Vec<Var>) -> coma::Expr,
) -> coma::Expr {
    let (floats, rest) = split_floats(lets, &fvs);
    let e = f(rest);
    e.with_(floats)
}

/// Splits `lets` using `fvs`, it finds a partition such that no variables in `fvs` occur in the second component.
/// `fvs` represents the `fvs` which are "defined here"
fn split_floats(mut lets: Vec<Var>, here: &HashSet<QName>) -> (Vec<Var>, Vec<Var>) {
    let mut max_ix: isize = -1;
    for (ix, v) in lets.iter().enumerate() {
        if here.contains(&v.0.clone().into()) {
            max_ix = ix as isize;
        }
    }

    let rest = lets.split_off(((max_ix + 1) as usize).min(lets.len()));
    let floats = lets;

    (floats, rest)
}
