fof(premise_1,axiom,(! [X] : (bornin(X, multiplebirth) => spendtimeplayingwith(X, sibling)))).
fof(premise_2,axiom,(! [X] : ((? [Y] : (sibling(X, Y) & borntogether(Y))) => bornin(X, multiplebirth)))).
fof(premise_3,axiom,(! [X] : (complainaboutoften(X, annoyingsiblings) => (? [Y] : (sibling(X, Y) & borntogether(Y)))))).
fof(premise_4,axiom,(! [X] : (liveat(X, home) => ~(livewith(X, strangers))))).
fof(premise_5,axiom,(! [X] : (spendtimeplayingwith(X, sibling) => liveat(X, home)))).
fof(premise_6,axiom,~(((bornin(luke, multiplebirth) | livewith(luke, strangers)) & ~((bornin(luke, multiplebirth) & livewith(luke, strangers)))))).
fof(conclusion_negated,conjecture,~(complainaboutoften(luke, annoyingsiblings))).
