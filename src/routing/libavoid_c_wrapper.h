#ifndef LIBAVOID_C_WRAPPER_H
#define LIBAVOID_C_WRAPPER_H

#ifdef __cplusplus
extern "C" {
#endif

// Opaque pointer types
typedef void* AvoidRouter;
typedef void* AvoidShape;
typedef void* AvoidConnector;
typedef unsigned int AvoidConnectorId;
typedef unsigned int AvoidShapeId;

// Simple point structure
typedef struct {
    double x;
    double y;
} AvoidPoint;

// Simple rectangle structure
typedef struct {
    double min_x;
    double min_y; 
    double max_x;
    double max_y;
} AvoidRectangle;

// Router creation and destruction
AvoidRouter avoid_router_new(unsigned int flags);
void avoid_router_delete(AvoidRouter router);

// Add obstacles (shapes)
AvoidShapeId avoid_router_add_shape(AvoidRouter router, AvoidRectangle rect);
void avoid_router_delete_shape(AvoidRouter router, AvoidShapeId shape_id);

// Add connectors
AvoidConnectorId avoid_router_add_connector(AvoidRouter router, AvoidPoint start, AvoidPoint end);
void avoid_router_delete_connector(AvoidRouter router, AvoidConnectorId conn_id);

// Process routing
void avoid_router_process_transaction(AvoidRouter router);

// Get route for a connector
int avoid_router_get_route_points(AvoidRouter router, AvoidConnectorId conn_id, AvoidPoint** points);
void avoid_free_points(AvoidPoint* points);

#ifdef __cplusplus
}
#endif

#endif // LIBAVOID_C_WRAPPER_H