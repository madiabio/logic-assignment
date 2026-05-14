fof(premise_1,axiom,musicpiece(symphony9)).
fof(premise_2,axiom,(! [X] : (musicpiece(X) => (? [Y] : (composer(Y) & write(Y, X)))))).
fof(premise_3,axiom,writtenby(symphony9, beethoven)).
fof(premise_4,axiom,premiered(viennamusicsociety, symphony9)).
fof(premise_5,axiom,orchestra(viennamusicsociety)).
fof(premise_6,axiom,lead(beethoven, viennamusicsociety)).
fof(premise_7,axiom,(! [X] : (orchestra(X) => ((? [Y, Conductor] : y) & lead(y, X))))).
fof(conclusion_negated,conjecture,~((? [X] : (? [Y] : ((orchestra(X) & musicpiece(Y)) & premiered(X, Y)))))).
