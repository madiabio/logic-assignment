fof(premise_1,axiom,(? [X] : ((show(X) & airingaton(X, c_8pmmonday)) & givenouton(X, rose, tv)))).
fof(premise_2,axiom,(! [X] : ((show(X) & givenoutonat(rose, tv, X)) => thebachelor(X)))).
fof(premise_3,axiom,(! [X] : (thebachelor(X) => portray(X, lifeofrealpeople)))).
fof(premise_4,axiom,(! [X] : (portray(X, liveofrealpeople) => realitytvshow(X)))).
fof(premise_5,axiom,show(breakingbad)).
fof(premise_6,axiom,~(realitytvshow(breakingbad))).
fof(conclusion_negated,conjecture,~((! [X] : (! [Y] : ((((mondayat8pm(X) & rose(Y)) & givenouton(Y, tv)) & on(tv, X)) & from(Y, breakingbad)))))).
