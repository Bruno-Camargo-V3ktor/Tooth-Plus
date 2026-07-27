BEGIN TRANSACTION;

DEFINE TABLE appointment SCHEMAFULL;
DEFINE FIELD start_time ON appointment TYPE datetime;
DEFINE FIELD end_time ON appointment TYPE datetime;
DEFINE FIELD status ON appointment TYPE string ASSERT $value INSIDE ['scheduled', 'confirmed', 'in_progress', 'completed', 'cancelled'];
DEFINE FIELD patient ON appointment TYPE record<patient>;
DEFINE FIELD dentist ON appointment TYPE record<user>;
DEFINE FIELD chair ON appointment TYPE record<chair>;
DEFINE FIELD clinic ON appointment TYPE record<clinic>;
DEFINE FIELD notes ON appointment TYPE option<string>;


DEFINE TABLE anamnesis SCHEMAFULL;
DEFINE FIELD patient ON anamnesis TYPE record<patient>;
DEFINE FIELD clinic ON anamnesis TYPE record<clinic>;
DEFINE FIELD answered_at ON anamnesis TYPE datetime VALUE time::now();
DEFINE FIELD answers ON anamnesis TYPE array<object>;
DEFINE FIELD answers.*.question ON anamnesis TYPE string;
DEFINE FIELD answers.*.response ON anamnesis TYPE string;

COMMIT TRANSACTION;
