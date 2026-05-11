fof(premise_1,axiom,(! [X] : (businessorganization(X) => legalentity(X)))).
fof(premise_2,axiom,(! [X] : (company(X) => businessorganization(X)))).
fof(premise_3,axiom,(! [X] : (privatecompany(X) => company(X)))).
fof(premise_4,axiom,(! [X] : (legalentity(X) => createdunderlaw(X)))).
fof(premise_5,axiom,(! [X] : (legalentity(X) => legalobligation(X)))).
fof(premise_6,axiom,(createdunderlaw(harvardweeklybookclub) => ~(privatecompany(harvardweeklybookclub)))).
fof(conclusion,conjecture,(legalobligation(harvardweeklybookclub) & privatecompany(harvardweeklybookclub))).
