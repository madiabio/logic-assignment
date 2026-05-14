fof(premise_1,axiom,(! [X] : (machinelearningalgorithm(X) => ((supervisedlearningalgorithm(X) | unsupervisedlearningalgorithm(X)) | reinforcementlearningalgorithm(X))))).
fof(premise_2,axiom,(! [X] : (unsupervisedlearningalgorithm(X) => ~(require(X, labeleddata))))).
fof(premise_3,axiom,(! [X] : (trainedwith(stateofthearttextsummarizationmodel, X) => machinelearningalgorithm(X)))).
fof(premise_4,axiom,(! [X] : (reinforcementlearningalgorithm(X) => ~(trainedwith(stateofthearttextsummarizationmodel, X))))).
fof(premise_5,axiom,(! [X] : ((machinelearningalgorithm(X) & trainedwith(stateofthearttextsummarizationmodel, X)) => require(X, labeleddata)))).
fof(conclusion,conjecture,(? [X] : (supervisedlearningalgorithm(X) & trainedwith(stateofthearttextsummarizationmodel, X)))).
