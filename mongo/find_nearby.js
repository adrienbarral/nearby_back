db.available.aggregate([ 
    {
        $geoNear: {
            near: { type: "Point", coordinates: [ 6.0000000 , 43.000000 ] },
            distanceField: "distance",
            maxDistance: 20000,
            query: {"contacts_phone_number_hash": "John Lenine"},
            spherical: true
         }
    },
    {
        $project: {
            "phone_number_hash": 1,
            "distance": 1
        }
    }
]);