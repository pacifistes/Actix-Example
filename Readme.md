# Vehicle Booking API

A Rust-based REST API for vehicle booking management built with Actix-web and MongoDB.

## 🚀 Quick Start

### Prerequisites
- Docker and Docker Compose
- Git

### Running the Project

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd Actix-Example
   ```

2. **Start the services**
   ```bash
   ./build.sh
    then   
   ./scripts/docker.sh start
   ```

3. **Start the API server**
   ```bash
   ./scripts/api.sh run
   ```

### 🌐 Access Points

- **API Server**: http://localhost:8080
- **MongoDB**: localhost:27017
- **Mongo Express** (Database UI): http://localhost:8081
  - Username: `root`
  - Password: `example`

### 🧪 Test the API

```bash
# Test the root endpoint
curl http://localhost:8080/

# Check MongoDB health
curl http://localhost:8080/health/mongodb

# Test protected endpoint (requires API key)
curl -H "X-API-Key: Admin" http://localhost:8080/identity
```

### 🛠️ Development Commands

```bash
# Stop containers
./scripts/docker.sh stop

# Build the application
./scripts/api.sh build

# Run tests
./scripts/api.sh test

# Format code
./scripts/api.sh fmt

# Run clippy linter
./scripts/api.sh clippy
```

---

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
