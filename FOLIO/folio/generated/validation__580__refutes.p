fof(premise_1,axiom,(! [X] : (plunger(X) => suck(X)))).
fof(premise_2,axiom,(! [X] : (vacuum(X) => suck(X)))).
fof(premise_3,axiom,(! [X] : (vampire(X) => suck(X)))).
fof(premise_4,axiom,vacuum(space)).
fof(premise_5,axiom,(householdappliance(duster) & ~(suck(duster)))).
fof(conclusion_negated,conjecture,~((! [X] : (householdapp(X) => suck(X))))).
