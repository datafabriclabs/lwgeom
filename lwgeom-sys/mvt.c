/**********************************************************************
 *
 * PostGIS - Spatial Types for PostgreSQL
 * http://postgis.net
 *
 * PostGIS is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 2 of the License, or
 * (at your option) any later version.
 *
 * PostGIS is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with PostGIS.  If not, see <http://www.gnu.org/licenses/>.
 *
 **********************************************************************
 *
 * Copyright (C) 2016-2017 Björn Harrtell <bjorn@wololo.org>
 *
 **********************************************************************/

#include <stdlib.h>
#include <assert.h>

#include "mvt.h"
#include "lwgeom_wagyu.h"

#define Max(x, y) ((x) > (y) ? (x) : (y))

/* For a given geometry, look for the highest dimensional basic type, that is,
 * point, line or polygon */
static uint8_t
lwgeom_get_basic_type(LWGEOM *geom)
{
	switch(geom->type)
	{
	case POINTTYPE:
	case LINETYPE:
	case POLYGONTYPE:
		return geom->type;
	case TRIANGLETYPE:
		return POLYGONTYPE;
	case MULTIPOINTTYPE:
	case MULTILINETYPE:
	case MULTIPOLYGONTYPE:
		return geom->type - 3; /* Based on LWTYPE positions */
	case COLLECTIONTYPE:
	case TINTYPE:
	{
		uint32_t i;
		uint8_t type = 0;
		LWCOLLECTION *g = (LWCOLLECTION*)geom;
		for (i = 0; i < g->ngeoms; i++)
		{
			LWGEOM *sg = g->geoms[i];
			type = Max(type, lwgeom_get_basic_type(sg));
		}
		return type;
	}
	default:
		exit(1);
	}
}


/**
 * In place process a collection to find a concrete geometry
 * object and expose that as the actual object. Will some
 * geom be lost? Sure, but your MVT renderer couldn't
 * draw it anyways.
 */
static inline LWGEOM *
lwgeom_to_basic_type(LWGEOM *geom, uint8_t original_type)
{
	LWGEOM *geom_out = geom;
	if (lwgeom_get_type(geom) == COLLECTIONTYPE)
	{
		LWCOLLECTION *g = (LWCOLLECTION*)geom;
		geom_out = (LWGEOM *)lwcollection_extract(g, original_type);
	}

	/* If a collection only contains 1 geometry return than instead */
	if (lwgeom_is_collection(geom_out))
	{
		LWCOLLECTION *g = (LWCOLLECTION *)geom_out;
		if (g->ngeoms == 1)
		{
			geom_out = g->geoms[0];
		}
	}

	geom_out->srid = geom->srid;
	return geom_out;
}

/* Clips a geometry using lwgeom_clip_by_rect. Might return NULL */
static LWGEOM *
mvt_unsafe_clip_by_box(LWGEOM *lwg_in, GBOX *clip_box)
{
	LWGEOM *geom_clipped;
	GBOX geom_box;

	gbox_init(&geom_box);
	FLAGS_SET_GEODETIC(geom_box.flags, 0);
	lwgeom_calculate_gbox(lwg_in, &geom_box);

	if (!gbox_overlaps_2d(&geom_box, clip_box))
	{
		return NULL;
	}

	if (gbox_contains_2d(clip_box, &geom_box))
	{
		return lwg_in;
	}

	geom_clipped = lwgeom_clip_by_rect(lwg_in, clip_box->xmin, clip_box->ymin, clip_box->xmax, clip_box->ymax);
	if (!geom_clipped || lwgeom_is_empty(geom_clipped))
		return NULL;
	return geom_clipped;
}

/* Clips a geometry for MVT using GEOS.
 * Does NOT work for polygons
 * Might return NULL
 */
static LWGEOM *
mvt_clip_and_validate_geos(LWGEOM *lwgeom, uint8_t basic_type, uint32_t extent, uint32_t buffer, bool clip_geom)
{
	LWGEOM *ng = lwgeom;
	assert(lwgeom->type != POLYGONTYPE);
	assert(lwgeom->type != MULTIPOLYGONTYPE);

	if (clip_geom)
	{
		gridspec grid = {0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0};
		GBOX bgbox;
		bgbox.xmax = bgbox.ymax = (double)extent + (double)buffer;
		bgbox.xmin = bgbox.ymin = -(double)buffer;
		bgbox.flags = 0;

		ng = mvt_unsafe_clip_by_box(ng, &bgbox);

		/* Make sure there is no pending float values (clipping can do that) */
		lwgeom_grid_in_place(ng, &grid);
	}

	return ng;
}

static LWGEOM *
mvt_clip_and_validate(LWGEOM *lwgeom, uint8_t basic_type, uint32_t extent, uint32_t buffer, bool clip_geom)
{
	GBOX clip_box = {0};
	LWGEOM *clipped_lwgeom;

	/* Wagyu only supports polygons. Default to geos for other types */
	lwgeom = lwgeom_to_basic_type(lwgeom, POLYGONTYPE);
	if (lwgeom->type != POLYGONTYPE && lwgeom->type != MULTIPOLYGONTYPE)
	{
		return mvt_clip_and_validate_geos(lwgeom, basic_type, extent, buffer, clip_geom);
	}

	if (!clip_geom)
	{
		/* With clipping disabled, we request a clip with the geometry bbox to force validation */
		lwgeom_calculate_gbox(lwgeom, &clip_box);
	}
	else
	{
		clip_box.xmax = clip_box.ymax = (double)extent + (double)buffer;
		clip_box.xmin = clip_box.ymin = -(double)buffer;
	}

	clipped_lwgeom = lwgeom_wagyu_clip_by_box(lwgeom, &clip_box);

	return clipped_lwgeom;
}

/**
 * Transform a geometry into vector tile coordinate space.
 *
 * Makes best effort to keep validity. Might collapse geometry into lower
 * dimension.
 *
 * NOTE: modifies in place if possible (not currently possible for polygons)
 */
LWGEOM *mvt_geom(LWGEOM *lwgeom, const GBOX *gbox, uint32_t extent, uint32_t buffer,
	bool clip_geom)
{
	AFFINE affine = {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};
	gridspec grid = {0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0};
	double width = gbox->xmax - gbox->xmin;
	double height = gbox->ymax - gbox->ymin;
	double fx, fy;
	const uint8_t basic_type = lwgeom_get_basic_type(lwgeom);
	int preserve_collapsed = LW_FALSE;

	/* Simplify it as soon as possible */
	lwgeom = lwgeom_to_basic_type(lwgeom, basic_type);

	/* Short circuit out on EMPTY */
	if (lwgeom_is_empty(lwgeom))
		return NULL;

	fx = extent / width;
	fy = -(extent / height);

	/* If geometry has disappeared, you're done */
	if (lwgeom_is_empty(lwgeom))
		return NULL;

	/* transform to tile coordinate space */
	affine.afac = fx;
	affine.efac = fy;
	affine.ifac = 1;
	affine.xoff = -gbox->xmin * fx;
	affine.yoff = -gbox->ymax * fy;
	lwgeom_affine(lwgeom, &affine);

	/* Snap to integer precision, removing duplicate points */
	lwgeom_grid_in_place(lwgeom, &grid);

	/* Remove points on straight lines */
	lwgeom_simplify_in_place(lwgeom, 0, preserve_collapsed);

	/* Remove duplicates in multipoints */
	if (lwgeom->type == MULTIPOINTTYPE)
		lwgeom_remove_repeated_points_in_place(lwgeom, 0);

	if (!lwgeom || lwgeom_is_empty(lwgeom))
		return NULL;

	lwgeom = mvt_clip_and_validate(lwgeom, basic_type, extent, buffer, clip_geom);
	if (!lwgeom || lwgeom_is_empty(lwgeom))
		return NULL;

	return lwgeom;
}
