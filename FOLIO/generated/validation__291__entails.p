fof(premise_1,axiom,(professionalwrestlingstable(diamondmine) & in(diamondmine, wwe))).
fof(premise_2,axiom,leads(roderickstrong, diamondmine)).
fof(premise_3,axiom,(includes(diamondmine, creedbrothers) & includes(diamondmine, ivynile))).
fof(premise_4,axiom,feuds(imperium, diamondmine)).
fof(conclusion,conjecture,(! [X] : ((professionalwrestlingstable(X) & includes(X, ivynile)) => ~(feuds(imperium, X))))).
