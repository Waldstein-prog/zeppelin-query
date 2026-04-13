use rusqlite::{Connection, Result, params};

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS airships (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            airship_type TEXT NOT NULL,
            manufacturer TEXT NOT NULL,
            country TEXT NOT NULL,
            first_flight TEXT,
            length_m REAL,
            volume_m3 REAL,
            max_speed_kmh REAL,
            passenger_capacity INTEGER,
            crew INTEGER,
            gas_type TEXT,
            fate TEXT
        );

        CREATE TABLE IF NOT EXISTS flights (
            id INTEGER PRIMARY KEY,
            airship_id INTEGER NOT NULL,
            departure_city TEXT NOT NULL,
            arrival_city TEXT NOT NULL,
            flight_date TEXT NOT NULL,
            duration_hours REAL,
            passengers INTEGER,
            cargo_kg REAL,
            purpose TEXT NOT NULL,
            FOREIGN KEY (airship_id) REFERENCES airships(id)
        );

        CREATE TABLE IF NOT EXISTS incidents (
            id INTEGER PRIMARY KEY,
            airship_id INTEGER NOT NULL,
            incident_date TEXT NOT NULL,
            location TEXT NOT NULL,
            description TEXT NOT NULL,
            casualties INTEGER DEFAULT 0,
            airship_survived INTEGER DEFAULT 1,
            FOREIGN KEY (airship_id) REFERENCES airships(id)
        );

        CREATE TABLE IF NOT EXISTS saved_queries (
            id INTEGER PRIMARY KEY,
            question TEXT NOT NULL,
            sql_query TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
    ")?;

    // Migration: add color column
    let has_color: bool = conn
        .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='saved_queries'")?
        .query_row([], |row| row.get::<_, String>(0))
        .map(|sql| sql.contains("color"))
        .unwrap_or(false);
    if !has_color {
        conn.execute_batch("ALTER TABLE saved_queries ADD COLUMN color TEXT DEFAULT NULL")?;
    }

    Ok(())
}

pub fn seed_data(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM airships", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }

    // === AIRSHIPS ===
    let airships = vec![
        (1, "LZ 1", "rigid", "Luftschiffbau Zeppelin", "Germany", "1900-07-02", 128.0, 11300.0, 28.0, 0, 5, "hydrogen", "Scrapped after 3 flights"),
        (2, "LZ 4", "rigid", "Luftschiffbau Zeppelin", "Germany", "1908-06-20", 136.0, 15000.0, 55.0, 0, 12, "hydrogen", "Destroyed in storm at Echterdingen 1908"),
        (3, "LZ 127 Graf Zeppelin", "rigid", "Luftschiffbau Zeppelin", "Germany", "1928-09-18", 236.6, 105000.0, 128.0, 20, 40, "hydrogen", "Scrapped 1940"),
        (4, "LZ 129 Hindenburg", "rigid", "Luftschiffbau Zeppelin", "Germany", "1936-03-04", 245.0, 200000.0, 135.0, 72, 40, "hydrogen", "Destroyed in fire at Lakehurst 1937"),
        (5, "LZ 130 Graf Zeppelin II", "rigid", "Luftschiffbau Zeppelin", "Germany", "1938-09-14", 245.0, 200000.0, 131.0, 72, 40, "hydrogen", "Scrapped 1940"),
        (6, "USS Akron (ZRS-4)", "rigid", "Goodyear-Zeppelin", "United States", "1931-09-23", 239.0, 184000.0, 130.0, 0, 89, "helium", "Crashed in storm 1933"),
        (7, "USS Macon (ZRS-5)", "rigid", "Goodyear-Zeppelin", "United States", "1933-04-21", 239.0, 184000.0, 130.0, 0, 89, "helium", "Crashed in storm 1935"),
        (8, "R100", "rigid", "Airship Guarantee Company", "United Kingdom", "1929-12-16", 219.0, 146000.0, 130.0, 100, 37, "hydrogen", "Scrapped 1931 after R101 disaster"),
        (9, "R101", "rigid", "Royal Airship Works", "United Kingdom", "1929-10-14", 237.0, 156000.0, 113.0, 100, 42, "hydrogen", "Crashed at Beauvais France 1930"),
        (10, "Italia", "semi-rigid", "Stabilimento Costruzioni Aeronautiche", "Italy", "1928-03-15", 105.0, 18500.0, 100.0, 0, 18, "hydrogen", "Crashed on North Pole expedition 1928"),
        (11, "Norge", "semi-rigid", "Stabilimento Costruzioni Aeronautiche", "Italy", "1926-03-29", 106.0, 18500.0, 100.0, 0, 16, "hydrogen", "Dismantled in Alaska 1926"),
        (12, "USS Los Angeles (ZR-3)", "rigid", "Luftschiffbau Zeppelin", "Germany", "1924-08-27", 200.0, 70000.0, 124.0, 0, 43, "helium", "Decommissioned 1932, scrapped 1939"),
        (13, "Goodyear Blimp Columbia", "blimp", "Goodyear", "United States", "1969-01-01", 58.0, 5380.0, 80.0, 6, 2, "helium", "Retired 1986"),
        (14, "Zeppelin NT 07", "semi-rigid", "Zeppelin Luftschifftechnik", "Germany", "1997-09-18", 75.0, 8225.0, 125.0, 12, 2, "helium", "Active - passenger flights"),
        (15, "Airlander 10", "hybrid", "Hybrid Air Vehicles", "United Kingdom", "2016-08-17", 92.0, 38000.0, 148.0, 48, 4, "helium", "In development"),
    ];

    for a in &airships {
        conn.execute(
            "INSERT INTO airships (id, name, airship_type, manufacturer, country, first_flight, length_m, volume_m3, max_speed_kmh, passenger_capacity, crew, gas_type, fate) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            params![a.0, a.1, a.2, a.3, a.4, a.5, a.6, a.7, a.8, a.9, a.10, a.11, a.12],
        )?;
    }

    // === FLIGHTS ===
    let flights: Vec<(i32, i32, &str, &str, &str, f64, i32, f64, &str)> = vec![
        (1, 3, "Friedrichshafen", "Lakehurst", "1928-10-11", 111.0, 20, 500.0, "commercial"),
        (2, 3, "Lakehurst", "Friedrichshafen", "1928-10-29", 71.0, 18, 300.0, "commercial"),
        (3, 3, "Friedrichshafen", "Tokyo", "1929-08-15", 101.0, 20, 400.0, "exhibition"),
        (4, 3, "Friedrichshafen", "Recife", "1930-05-18", 62.0, 18, 600.0, "commercial"),
        (5, 3, "Recife", "Friedrichshafen", "1930-06-01", 58.0, 16, 200.0, "commercial"),
        (6, 3, "Friedrichshafen", "Lakehurst", "1929-08-01", 95.0, 20, 450.0, "exhibition"),
        (7, 3, "Lakehurst", "Friedrichshafen", "1929-08-10", 67.0, 19, 350.0, "exhibition"),
        (8, 3, "Friedrichshafen", "Recife", "1931-03-20", 60.0, 20, 700.0, "commercial"),
        (9, 3, "Friedrichshafen", "Recife", "1932-04-10", 55.0, 18, 650.0, "commercial"),
        (10, 3, "Recife", "Friedrichshafen", "1932-04-25", 57.0, 17, 300.0, "commercial"),
        (11, 4, "Frankfurt", "Lakehurst", "1936-05-06", 61.0, 50, 1200.0, "commercial"),
        (12, 4, "Lakehurst", "Frankfurt", "1936-05-17", 53.0, 48, 800.0, "commercial"),
        (13, 4, "Frankfurt", "Lakehurst", "1936-06-17", 59.0, 55, 1100.0, "commercial"),
        (14, 4, "Lakehurst", "Frankfurt", "1936-07-02", 55.0, 52, 900.0, "commercial"),
        (15, 4, "Frankfurt", "Recife", "1936-03-31", 62.0, 60, 1500.0, "commercial"),
        (16, 4, "Recife", "Frankfurt", "1936-04-10", 58.0, 45, 400.0, "commercial"),
        (17, 4, "Frankfurt", "Lakehurst", "1936-08-10", 63.0, 65, 1300.0, "commercial"),
        (18, 4, "Frankfurt", "Lakehurst", "1937-05-03", 60.0, 36, 1000.0, "commercial"),
        (19, 8, "Cardington", "Montreal", "1930-07-29", 78.0, 44, 800.0, "test"),
        (20, 8, "Montreal", "Cardington", "1930-08-13", 57.0, 42, 600.0, "test"),
        (21, 9, "Cardington", "Ismailia", "1930-10-04", 34.0, 54, 1000.0, "test"),
        (22, 11, "Rome", "Ny-Ålesund", "1926-04-10", 72.0, 0, 100.0, "exhibition"),
        (23, 11, "Ny-Ålesund", "Teller", "1926-05-11", 70.0, 0, 50.0, "exhibition"),
        (24, 10, "Milan", "Ny-Ålesund", "1928-04-15", 48.0, 0, 200.0, "exhibition"),
        (25, 10, "Ny-Ålesund", "North Pole", "1928-05-23", 20.0, 0, 50.0, "exhibition"),
        (26, 12, "Friedrichshafen", "Lakehurst", "1924-10-12", 81.0, 0, 400.0, "military"),
        (27, 6, "Lakehurst", "San Diego", "1932-05-08", 36.0, 0, 0.0, "military"),
        (28, 6, "Lakehurst", "Panama Canal Zone", "1933-01-09", 42.0, 0, 0.0, "military"),
        (29, 7, "Lakehurst", "Sunnyvale", "1933-10-15", 48.0, 0, 0.0, "military"),
        (30, 14, "Friedrichshafen", "Friedrichshafen", "2001-05-15", 1.5, 12, 0.0, "commercial"),
        (31, 14, "Friedrichshafen", "Friedrichshafen", "2005-07-20", 1.0, 10, 0.0, "commercial"),
        (32, 14, "Friedrichshafen", "Friedrichshafen", "2010-09-12", 1.5, 11, 0.0, "commercial"),
        (33, 3, "Friedrichshafen", "Recife", "1933-06-15", 54.0, 20, 700.0, "commercial"),
        (34, 3, "Recife", "Friedrichshafen", "1933-07-01", 56.0, 19, 400.0, "commercial"),
        (35, 4, "Frankfurt", "Rio de Janeiro", "1936-10-05", 65.0, 58, 1400.0, "commercial"),
        (36, 4, "Rio de Janeiro", "Frankfurt", "1936-10-20", 60.0, 42, 500.0, "commercial"),
        (37, 5, "Frankfurt", "Frankfurt", "1938-09-20", 6.0, 0, 0.0, "test"),
        (38, 5, "Frankfurt", "Frankfurt", "1939-07-15", 8.0, 0, 0.0, "military"),
        (39, 13, "Akron", "New York", "1972-06-01", 5.0, 6, 0.0, "commercial"),
        (40, 13, "Los Angeles", "Los Angeles", "1980-01-20", 3.0, 4, 0.0, "commercial"),
    ];

    for f in &flights {
        conn.execute(
            "INSERT INTO flights (id, airship_id, departure_city, arrival_city, flight_date, duration_hours, passengers, cargo_kg, purpose) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![f.0, f.1, f.2, f.3, f.4, f.5, f.6, f.7, f.8],
        )?;
    }

    // === INCIDENTS ===
    let incidents: Vec<(i32, i32, &str, &str, &str, i32, bool)> = vec![
        (1, 1, "1900-07-02", "Lake Constance, Germany", "Hard landing on first flight due to inadequate control surfaces", 0, true),
        (2, 2, "1908-08-05", "Echterdingen, Germany", "Destroyed by fire during storm while moored. Public donations funded replacement.", 0, false),
        (3, 4, "1937-05-06", "Lakehurst, New Jersey, USA", "Caught fire while landing, destroyed in 34 seconds. Cause debated: static discharge or sabotage.", 36, false),
        (4, 6, "1933-04-04", "Atlantic Ocean off New Jersey", "Structural failure in severe storm, crashed into ocean", 73, false),
        (5, 7, "1935-02-12", "Pacific Ocean off Point Sur, California", "Upper fin structural failure in storm, crashed into ocean", 2, false),
        (6, 9, "1930-10-05", "Beauvais, France", "Crashed into hillside in bad weather on maiden voyage to India. Ended British airship program.", 48, false),
        (7, 10, "1928-05-25", "Arctic Ocean near North Pole", "Crashed on return from North Pole. Rescue took 49 days.", 1, false),
        (8, 12, "1927-08-25", "Lakehurst, New Jersey, USA", "Nose caught by sudden gust, stood on tail vertically. No casualties, repaired.", 0, true),
        (9, 6, "1932-05-11", "Camp Kearny, San Diego", "Tail hit ground during landing attempt in crosswind. Minor damage, repaired.", 0, true),
        (10, 15, "2016-08-24", "Cardington, United Kingdom", "Hard landing on second test flight, nose damaged. Pilot minor injuries.", 0, true),
    ];

    for i in &incidents {
        conn.execute(
            "INSERT INTO incidents (id, airship_id, incident_date, location, description, casualties, airship_survived) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![i.0, i.1, i.2, i.3, i.4, i.5, i.6],
        )?;
    }

    // === SEED SAVED QUERIES ===
    let saved_count: i64 = conn.query_row("SELECT COUNT(*) FROM saved_queries", [], |r| r.get(0))?;
    if saved_count == 0 {
        let saved_queries: Vec<(&str, &str)> = vec![
            (
                "Incidenten per jaar",
                "SELECT substr(incident_date, 1, 4) AS jaar, COUNT(*) AS aantal, SUM(casualties) AS totaal_slachtoffers FROM incidents GROUP BY jaar ORDER BY jaar"
            ),
            (
                "Vluchten per luchtschip (pivot)",
                "SELECT a.name, COUNT(f.id) AS aantal_vluchten, SUM(f.passengers) AS totaal_passagiers, ROUND(AVG(f.duration_hours), 1) AS gem_duur_uren, SUM(f.cargo_kg) AS totaal_cargo_kg FROM airships a LEFT JOIN flights f ON a.id = f.airship_id GROUP BY a.id ORDER BY aantal_vluchten DESC"
            ),
            (
                "Luchtschepen met incidenten en hun lot",
                "SELECT a.name, a.country, a.gas_type, COUNT(i.id) AS aantal_incidenten, SUM(i.casualties) AS totaal_slachtoffers, a.fate FROM airships a INNER JOIN incidents i ON a.id = i.airship_id GROUP BY a.id ORDER BY totaal_slachtoffers DESC"
            ),
            (
                "Top routes op passagiersaantal",
                "SELECT departure_city || ' → ' || arrival_city AS route, COUNT(*) AS vluchten, SUM(passengers) AS totaal_passagiers FROM flights GROUP BY departure_city, arrival_city HAVING totaal_passagiers > 0 ORDER BY totaal_passagiers DESC"
            ),
        ];

        for (question, sql) in &saved_queries {
            conn.execute(
                "INSERT INTO saved_queries (question, sql_query) VALUES (?1, ?2)",
                params![question, sql],
            )?;
        }
    }

    Ok(())
}

pub fn get_schema_description() -> String {
    r#"Database schema for a historical airship/zeppelin database:

TABLE airships (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,              -- e.g. "LZ 129 Hindenburg", "USS Akron (ZRS-4)"
    airship_type TEXT NOT NULL,      -- "rigid", "semi-rigid", "blimp", or "hybrid"
    manufacturer TEXT NOT NULL,      -- e.g. "Luftschiffbau Zeppelin", "Goodyear-Zeppelin"
    country TEXT NOT NULL,           -- country of origin: "Germany", "United States", "United Kingdom", "Italy"
    first_flight TEXT,               -- date as YYYY-MM-DD
    length_m REAL,                   -- length in meters
    volume_m3 REAL,                  -- gas volume in cubic meters
    max_speed_kmh REAL,              -- maximum speed in km/h
    passenger_capacity INTEGER,      -- max passenger capacity (0 for military)
    crew INTEGER,                    -- crew size
    gas_type TEXT,                   -- "hydrogen" or "helium"
    fate TEXT                        -- what happened to the airship
);

TABLE flights (
    id INTEGER PRIMARY KEY,
    airship_id INTEGER NOT NULL,     -- references airships(id)
    departure_city TEXT NOT NULL,
    arrival_city TEXT NOT NULL,
    flight_date TEXT NOT NULL,        -- date as YYYY-MM-DD
    duration_hours REAL,             -- flight duration in hours
    passengers INTEGER,              -- number of passengers on this flight
    cargo_kg REAL,                   -- cargo weight in kilograms
    purpose TEXT NOT NULL,           -- "commercial", "military", "exhibition", or "test"
    FOREIGN KEY (airship_id) REFERENCES airships(id)
);

TABLE incidents (
    id INTEGER PRIMARY KEY,
    airship_id INTEGER NOT NULL,     -- references airships(id)
    incident_date TEXT NOT NULL,      -- date as YYYY-MM-DD
    location TEXT NOT NULL,
    description TEXT NOT NULL,
    casualties INTEGER DEFAULT 0,
    airship_survived INTEGER DEFAULT 1, -- 1 = yes, 0 = no (boolean)
    FOREIGN KEY (airship_id) REFERENCES airships(id)
);"#.to_string()
}
