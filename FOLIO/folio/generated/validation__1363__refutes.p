fof(premise_1,axiom,(! [X] : ((rabbit(X) & canbespottednear(X, campus)) => cute(X)))).
fof(premise_2,axiom,(? [X] : (turtle(X) & canbespottednear(X, campus)))).
fof(premise_3,axiom,(! [X] : (canbespottednear(X, campus) => ((rabbit(X) | squirrel(X)) & ~((rabbit(X) & squirrel(X))))))).
fof(premise_4,axiom,(! [X] : (skittish(X) => ~(calm(X))))).
fof(premise_5,axiom,(! [X] : ((squirrel(X) & canbespottednear(X, campus)) => skittish(X)))).
fof(premise_6,axiom,(canbespottednear(rockie, campus) & calm(rockie))).
fof(conclusion_negated,conjecture,~((turtle(rockie) | cute(rockie)))).
