fof(premise_1,axiom,(! [X] : ((film(X) & rated(X, adultsonly)) => canwatchwithout(children, X, guidancefromparents)))).
fof(premise_2,axiom,(! [X] : (((film(X) & contain(X, excessiveeroticcontent)) & contain(X, excessiveviolentcontent)) => ~(canwatchwithout(children, X, guidancefromparents))))).
fof(premise_3,axiom,(! [X] : ((film(X) & rated(X, generalaudience)) => appropriatefor(X, peopleofallages)))).
fof(premise_4,axiom,(! [X] : (((film(X) & familyfriendly(X)) & animated(X)) => rated(X, generalaudience)))).
fof(premise_5,axiom,(! [X] : ((film(X) & infrozenseries(X)) => (familyfriendly(X) & animated(X))))).
fof(premise_6,axiom,film(hachiadogstale)).
fof(premise_7,axiom,(((familyfriendly(hachiadogstale) & animated(hachiadogstale)) | rated(hachiadogstale, adultsonly)) & ~(((familyfriendly(hachiadogstale) & animated(hachiadogstale)) & rated(hachiadogstale, adultsonly))))).
fof(conclusion,conjecture,((contain(x, excessiveeroticcontent) & contain(x, excessiveviolentcontent)) | infrozenseries(excessiveviolentcontent))).
