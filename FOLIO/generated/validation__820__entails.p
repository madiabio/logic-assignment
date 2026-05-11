fof(premise_1,axiom,(! [X] : (growthstock(X) => boughttoearnprofitfrom(X, rapidpriceappreciation)))).
fof(premise_2,axiom,(! [X] : (boughttoearnprofitfrom(X, earnprofit, rapidpriceappreciation) => ~(suitablefor(X, retirementfund))))).
fof(premise_3,axiom,(? [X] : (stock(X) & growthstock(X)))).
fof(premise_4,axiom,(! [X] : (maturestock(X) => suitablefor(X, retirementfund)))).
fof(premise_5,axiom,maturestock(ko)).
fof(conclusion,conjecture,~(growthstock(ko))).
