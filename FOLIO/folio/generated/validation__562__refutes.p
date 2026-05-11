fof(premise_1,axiom,(cost(gre, c_205) & cheaper(c_205, c_300))).
fof(premise_2,axiom,(! [X] : ((applicantof(X, gre) & prove(X, economichardship)) => provideto(ets, financialaid, X)))).
fof(premise_3,axiom,(! [X] : ((livingin(X, singleparentfamily) | availableto(fewresources, X)) => prove(X, economichardship)))).
fof(premise_4,axiom,livingin(tom, singleparentfamily)).
fof(premise_5,axiom,(outofwork(tomsdad) & availableto(fewresources, tom))).
fof(premise_6,axiom,applicantof(tom, gre)).
fof(conclusion_negated,conjecture,~(~((? [X] : (? [Y] : (applicant(X, gre) & providesfinancialaidto(Y, X))))))).
