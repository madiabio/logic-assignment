fof(premise_1,axiom,(! [X] : (naturallanguageprocessingtask(X) => ((languagegenerationtask(X) | languageunderstandingtask(X)) & ~((languagegenerationtask(X) & languageunderstandingtask(X))))))).
fof(premise_2,axiom,(! [X] : ((naturallanguageprocessingtasks(X) & outputsequence(X, text)) => languagegenerationtask(X)))).
fof(premise_3,axiom,naturallanguageprocessingtask(machinetranslation)).
fof(premise_4,axiom,outputsequence(machinetranslation, text)).
fof(conclusion,conjecture,languagegenerationtask(machinetranslation)).
