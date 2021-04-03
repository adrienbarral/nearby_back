db.createCollection("available");

db.available.createIndex( { "contacts_phone_number_hash" : 1 } );
db.available.createIndex( { "location" : "2dsphere" } );
