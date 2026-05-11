fof(premise_1,axiom,(bornin(ailtonsilva, year1995) & commonlyknownas(ailtonsilva, ailton))).
fof(premise_2,axiom,(footballplayer(ailton) & loanedto(ailton, braga))).
fof(premise_3,axiom,((brazilian(ailtonsilva) & footballplayer(ailtonsilva)) & playfor(ailtonsilva, nautico))).
fof(premise_4,axiom,(footballclub(nautico) & footballclub(braga))).
fof(premise_5,axiom,footballclub(fluminense)).
fof(conclusion,conjecture,(! [X] : (footballclub(X) => ~(loanedto(ailton, X))))).
