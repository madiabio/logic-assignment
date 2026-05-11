fof(premise_1,axiom,(! [X] : (regularlyeat(X, salad) => (consciousabout(X, health) & consciousabout(X, eatingghabit))))).
fof(premise_2,axiom,(! [X] : (grewupin(X, health_consciouschildhoodhome) => regularlyeat(X, salad)))).
fof(premise_3,axiom,(! [X] : (fulfill(X, dailynutritionalintake) => grewupin(X, health_consciouschildhoodhome)))).
fof(premise_4,axiom,(! [X] : (disregard(X, physicalwellbeing) => ~((consciousabout(X, health) & consciousabout(X, eatinghabit)))))).
fof(premise_5,axiom,(! [X] : (visitdaily(X, gym) => fulfill(X, dailynutritionalintake)))).
fof(premise_6,axiom,~(((growupin(taylor, health_consciouschildhoodhome) | disregard(taylor, physicalwellbeing)) & ~((growupin(taylor, health_consciouschildhoodhome) & disregard(taylor, physicalwellbeing)))))).
fof(conclusion,conjecture,regularlyeat(taylor, salad)).
