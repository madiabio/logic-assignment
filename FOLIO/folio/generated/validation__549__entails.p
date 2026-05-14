fof(premise_1,axiom,(! [X] : ((internationalstudent(X) & in(X, unitedstates)) => ((f1visa(X) | j1visa(X)) & ~((f1visa(X) & j1visa(X))))))).
fof(premise_2,axiom,(! [X] : ((((internationalstudent(X) & in(X, unitedstates)) & f1visa(X)) & wanttoworkin(X, unitedstates)) => (apply(X, cpt) | apply(X, opt))))).
fof(premise_3,axiom,internationalstudent(mike)).
fof(premise_4,axiom,(wanttoworkin(x, unitedstates) => apply(mike, cpt))).
fof(conclusion,conjecture,j1visa(mike)).
