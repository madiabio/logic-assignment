fof(premise_1,axiom,(german(heinrichschmidt) & politician(heinrichschmidt))).
fof(premise_2,axiom,(member(heinrichschmidt, prussianstateparliament) & member(heinrichschmidt, nazireichstag))).
fof(conclusion_negated,conjecture,~((? [X] : (((german(X) & politician(X)) & member(X, prussianstateparliament)) & member(X, nazireichstag))))).
