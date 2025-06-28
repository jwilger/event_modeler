#include "libavoid_c_wrapper.h"
#include "libavoid/libavoid.h"
#include <vector>
#include <map>

// Keep track of shapes and connectors
static std::map<AvoidRouter, std::map<AvoidShapeId, Avoid::ShapeRef*>> g_shapes;
static std::map<AvoidRouter, std::map<AvoidConnectorId, Avoid::ConnRef*>> g_connectors;
static AvoidShapeId g_next_shape_id = 1;
static AvoidConnectorId g_next_connector_id = 1;

extern "C" {

AvoidRouter avoid_router_new(unsigned int flags) {
    Avoid::Router* router = new Avoid::Router(flags);
    return static_cast<AvoidRouter>(router);
}

void avoid_router_delete(AvoidRouter router) {
    if (!router) return;
    
    // Clean up our tracking maps
    g_shapes.erase(router);
    g_connectors.erase(router);
    
    Avoid::Router* r = static_cast<Avoid::Router*>(router);
    delete r;
}

AvoidShapeId avoid_router_add_shape(AvoidRouter router, AvoidRectangle rect) {
    if (!router) return 0;
    
    Avoid::Router* r = static_cast<Avoid::Router*>(router);
    
    // Create rectangle from our C struct
    Avoid::Rectangle avoid_rect(
        Avoid::Point(rect.min_x, rect.min_y),
        Avoid::Point(rect.max_x, rect.max_y)
    );
    
    // Create shape
    Avoid::ShapeRef* shape = new Avoid::ShapeRef(r, avoid_rect);
    
    // Store with ID
    AvoidShapeId id = g_next_shape_id++;
    g_shapes[router][id] = shape;
    
    return id;
}

void avoid_router_delete_shape(AvoidRouter router, AvoidShapeId shape_id) {
    if (!router) return;
    
    auto router_shapes = g_shapes.find(router);
    if (router_shapes != g_shapes.end()) {
        auto shape_it = router_shapes->second.find(shape_id);
        if (shape_it != router_shapes->second.end()) {
            delete shape_it->second;
            router_shapes->second.erase(shape_it);
        }
    }
}

AvoidConnectorId avoid_router_add_connector(AvoidRouter router, AvoidPoint start, AvoidPoint end) {
    if (!router) return 0;
    
    Avoid::Router* r = static_cast<Avoid::Router*>(router);
    
    // Create connector with endpoints
    Avoid::ConnRef* conn = new Avoid::ConnRef(r);
    
    // Set endpoints
    Avoid::ConnEnd src_end(Avoid::Point(start.x, start.y));
    Avoid::ConnEnd dst_end(Avoid::Point(end.x, end.y));
    conn->setEndpoints(src_end, dst_end);
    
    // Store with ID
    AvoidConnectorId id = g_next_connector_id++;
    g_connectors[router][id] = conn;
    
    return id;
}

void avoid_router_delete_connector(AvoidRouter router, AvoidConnectorId conn_id) {
    if (!router) return;
    
    auto router_conns = g_connectors.find(router);
    if (router_conns != g_connectors.end()) {
        auto conn_it = router_conns->second.find(conn_id);
        if (conn_it != router_conns->second.end()) {
            delete conn_it->second;
            router_conns->second.erase(conn_it);
        }
    }
}

void avoid_router_process_transaction(AvoidRouter router) {
    if (!router) return;
    
    Avoid::Router* r = static_cast<Avoid::Router*>(router);
    r->processTransaction();
}

int avoid_router_get_route_points(AvoidRouter router, AvoidConnectorId conn_id, AvoidPoint** points) {
    if (!router || !points) return 0;
    
    auto router_conns = g_connectors.find(router);
    if (router_conns == g_connectors.end()) return 0;
    
    auto conn_it = router_conns->second.find(conn_id);
    if (conn_it == router_conns->second.end()) return 0;
    
    Avoid::ConnRef* conn = conn_it->second;
    const Avoid::PolyLine& route = conn->displayRoute();
    
    if (route.size() == 0) return 0;
    
    // Allocate array for points
    *points = new AvoidPoint[route.size()];
    
    // Copy points
    for (size_t i = 0; i < route.size(); ++i) {
        (*points)[i].x = route.at(i).x;
        (*points)[i].y = route.at(i).y;
    }
    
    return static_cast<int>(route.size());
}

void avoid_free_points(AvoidPoint* points) {
    delete[] points;
}

} // extern "C"