# Vehicle Booking API

## 🔑 Authentication

Authentication is handled via **API Key**.
Available API Keys:

* `Admin`
* `CarManager`
* `MotorbikeManager`
* `Customer1`
* `Customer2`

Each role has specific permissions as described below.

---

## 👥 Roles & Permissions

* **Admin**: full access (manage vehicles and bookings).
* **CarManager / MotorbikeManager**: manage vehicles and bookings of their category.
* **Customer**: can only create and view their own bookings.

---

## 🚗 Resource: Vehicles

### Structure

```json
Vehicle {
  "id": "...",
  "brand": "...",
  "type": "CAR" | "MOTORBIKE",
  "metadata": { ... },
  "description": "...",
  "price_by_day": 50,
  "year_of_production": 2021
}
```

#### Types

* **CAR**

  ```json
  {
    "seats": 5,
    "model": "...",
    "gearbox": "MANUAL" | "AUTOMATIC",
    "fuelType": "PETROL" | "DIESEL" | "ELECTRIC"
  }
  ```
* **MOTORBIKE**

  ```json
  {
    "engineCc": 650,
    "hasSidecar": false
  }
  ```

---

### Endpoints

#### `POST /vehicles` (Admin)

* Add a new vehicle.
* Validation:

  * `description` ≤ 250 characters
  * If `brand = Tesla` → `fuelType` must be `ELECTRIC`

#### `GET /vehicles` (All)

* Retrieve list of vehicles.
* Supports **filters and pagination**.
* Custom deserialization: filters and sorting converted into hashmap.

#### `PATCH /vehicles/{id}` (Admin, CarManager, MotorbikeManager)

* Update vehicle data.
* Validation: check that the user has permission for this vehicle type.

#### `GET /vehicles/{id}/bookings` (Admin, CarManager, MotorbikeManager)

* Retrieve all bookings for a vehicle.
* Supports sorting by `from_date` or `order_date`.

---

## 📅 Resource: Bookings

### Structure

```json
Booking {
  "id": "...",
  "vehicle_id": "...",
  "from_date": "2025-08-01",
  "to_date": "2025-08-10",
  "status": "PENDING" | "CONFIRMED" | "REJECTED" | "CANCELLED",
  "reason": "..." // only if CANCELLED
}
```

---

### Endpoints

#### `POST /bookings` (Customer)

* Create a booking.
* Validation:

  * Vehicle must exist.
  * No overlapping booking allowed for the same period.

#### `GET /bookings` (Customer, Admin, Managers)

* **Customer**: only sees their own bookings.
* **Admin / Managers**: can view all bookings.

#### `PATCH /bookings/{id}` (Admin, CarManager, MotorbikeManager, Customer)

* Update a booking (change status, cancel, etc.).
* Validation: booking must exist + user must have permission.

---
