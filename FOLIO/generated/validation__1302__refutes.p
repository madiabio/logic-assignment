fof(premise_1,axiom,(! [X] : (alien(X) => extraterrestrial(X)))).
fof(premise_2,axiom,(! [X] : (from(X, mars) => alien(X)))).
fof(premise_3,axiom,(! [X] : (extraterrestrial(X) => ~(human(X))))).
fof(premise_4,axiom,(! [X] : ((highlyintelligentbeing(X) & from(X, earth)) => human(X)))).
fof(premise_5,axiom,highlyintelligentbeing(marvin)).
fof(premise_6,axiom,~(((from(marvin, earth) | from(marvin, mars)) & ~((from(marvin, earth) & from(marvin, mars)))))).
fof(premise_7,axiom,(~(from(marvin, earth)) => extraterrestrial(marvin))).
fof(conclusion_negated,conjecture,~((~(human(marvin)) & ~(from(marvin, mars))))).
