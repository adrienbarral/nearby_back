db.available.deleteMany({});

let user = {
    "phone_number_hash": "John Lenine",
    "location": {
        "type": "Point",
        "coordinates": [6.00000,43.000000]
    },
    "contacts_phone_number_hash": ["Sylvester Staline","Didier CrouteChef","Hugo Chat Vez"],
    "available_until": "2021-03-25T12:00:00+00:00"
}
db.available.insertOne(user);

user = {
    "phone_number_hash": "Sylverster Staline",
    "location": {
        "type": "Point",
        "coordinates": [6.00001,43.000001]
    },
    "contacts_phone_number_hash": ["John Lenine","Didier CrouteChef","Hugo Chat Vez"],
    "available_until": "2021-03-25T12:00:00+00:00"
}
db.available.insertOne(user);

user = {
    "phone_number_hash": "Didier CrouteChef",
    "location": {
        "type": "Point",
        "coordinates": [5.00000,43.000000]
    },
    "contacts_phone_number_hash": ["John Lenine","Sylverster Staline","Hugo Chat Vez"],
    "available_until": "2021-03-25T12:00:00+00:00"
}

user = {
    "phone_number_hash": "Unknown Man",
    "location": {
        "type": "Point",
        "coordinates": [6.00000,43.000000]
    },
    "contacts_phone_number_hash": ["Unknown friend"],
    "available_until": "2021-03-25T12:00:00+00:00"

}
db.available.insertOne(user);

/**
 * So here, if we are Joh Lenine, and we search for friends nearby we should find : 
 *   * Sylverster Staline (he is near John, and they know each other).
 * !! Unknwon man is also very close to John, but John doesn't know him, so it must not be
 * returned !
 */ 
  