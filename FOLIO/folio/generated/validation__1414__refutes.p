fof(premise_1,axiom,(! [X] : (canregistertovotein(X, unitedstates) => canparticipatein(X, c_2024unitedstateselection)))).
fof(premise_2,axiom,(! [X] : (have(X, unitedstatescitizenship) => canregistertovotein(X, unitedstates)))).
fof(premise_3,axiom,(! [X] : (have(X, unitedstatescitizenship) | have(X, taiwanesecitizenship)))).
fof(premise_4,axiom,(! [X] : ((russian(X) & federationofficial(X)) => ~(have(X, taiwanesecitizenship))))).
fof(premise_5,axiom,(~(have(vladimir, taiwanesecitizenship)) & ~(managerat(vladimir, gazprom)))).
fof(premise_6,axiom,((russian(ekaterina) & federationofficial(ekaterina)) | canregistertovotein(ekaterina, unitedstates))).
fof(conclusion_negated,conjecture,~((canregistertovotein(ekaterina, unitedstates) & canparticipatein(vladimir, c_2024unitedstateselection)))).
