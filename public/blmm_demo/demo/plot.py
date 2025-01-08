import numpy as np
import nibabel as nib
import plotly.graph_objects as go
import os

def plot_brain_surface(data_fnames, geom_fnames, volume_number=0, dtype=np.float32):
    
    # Convert single filename inputs to lists
    if not isinstance(data_fnames, list):
        data_fnames = [data_fnames]
    if not isinstance(geom_fnames, list):
        geom_fnames = [geom_fnames]
    
    # Extracting filename without extension
    title_name = os.path.splitext(os.path.basename(data_fnames[0]))[0]
    
    # Check if dtype is correct
    if title_name in ['blmm_vox_n', 'blmm_vox_mask', 'blmm_vox_edf']:
        dtype = np.int32
    
    # Check if the number of data files and geometry files are the same
    if len(data_fnames) != len(geom_fnames):
        raise ValueError("The number of data files must be equal to the number of geometry files.")

    # Find global minimum and maximum intensity values
    global_min, global_max = float('inf'), float('-inf')
    for data_fname, geom_fname in zip(data_fnames, geom_fnames):
        
        # Load a surface file
        surface = nib.freesurfer.io.read_geometry(geom_fname)
        vertices, faces = surface

        # Number of vertices in volume
        n_verts_in_vol = len(vertices)
        
        # Load in data 
        data = np.memmap(data_fname, dtype=dtype, mode='r')
        data = data.reshape(n_verts_in_vol, data.size // n_verts_in_vol)
        
        # Get min/max for colorbar
        data_min = np.min(data[:, volume_number])
        data_max = np.max(data[:, volume_number])
        global_min = min(global_min, data_min)
        global_max = max(global_max, data_max)

    # Create an empty figure
    fig = go.Figure()

    for data_fname, geom_fname in zip(data_fnames, geom_fnames):
        # Load a surface file
        surface = nib.freesurfer.io.read_geometry(geom_fname)
        vertices, faces = surface

        # Number of vertices in volume
        n_verts_in_vol = len(vertices)

        # Read in data
        data = np.memmap(data_fname, dtype=dtype, mode='r')
        data = data.reshape(n_verts_in_vol, data.size // n_verts_in_vol)

        # Creating a Plotly trace for the surface
        trace = go.Mesh3d(
            x=vertices[:, 0],  # X coordinates of vertices
            y=vertices[:, 1],  # Y coordinates of vertices
            z=vertices[:, 2],  # Z coordinates of vertices
            i=faces[:, 0],     # Indices of vertices forming the triangles (i, j, k)
            j=faces[:, 1],
            k=faces[:, 2],
            intensity=data[:, volume_number],      # Color intensity based on data array
            colorscale='Viridis',  # Color scale
            cmin=global_min,  # Minimum intensity value for consistent color scale
            cmax=global_max,  # Maximum intensity value for consistent color scale
            name=os.path.splitext(os.path.basename(data_fname))[0],  # Add name to the legend
            customdata=np.round(data[:, volume_number], 2),
            showscale=False,  # Show color scale only for the last trace
            hoverinfo='text',                    # Show text on hover
            hovertemplate=(
                f'{os.path.splitext(os.path.basename(data_fname))[0]}<br>' +
                'X: %{x}<br>Y: %{y}<br>Z: %{z}<br>' +
                'Intensity: %{customdata}<extra></extra>'  # Format of the hover text
            )
        )

        # Add trace to the figure
        fig.add_trace(trace)

    # Show color scale on the last trace
    fig.data[-1].showscale = True

    # Updating layout to hide axes and add title
    fig.update_layout(
        title=f"Brain Surface Visualization: {title_name}, Volume {str(volume_number)}",
        scene=dict(
            xaxis=dict(showbackground=False, showticklabels=False, showgrid=False, zeroline=False, showspikes=False),
            yaxis=dict(showbackground=False, showticklabels=False, showgrid=False, zeroline=False, showspikes=False),
            zaxis=dict(showbackground=False, showticklabels=False, showgrid=False, zeroline=False, showspikes=False)
        ),
        scene_camera=dict(
            eye=dict(x=1.5, y=1.5, z=1.5)
        ),
        legend_title_text='Data Files'
    )

    # Showing the figure
    fig.show()

    
def get_fname(analysis, hemisphere, image):
    
    # Work out if we are looking at left or right hemisphere
    if hemisphere.lower() == 'left':
        lr_str = 'lh'
    elif hemisphere.lower() == 'right':
        lr_str = 'rh'
    else:
        raise ValueError('Please specify left or right for hemisphere')
        
    # Construct directory
    directory = 'results_' + lr_str + '_' + analysis
    
    # Construct filename
    fname = 'blmm_vox_' + image + '.dat'
    
    # Return full filename
    return(os.path.join(os.getcwd(), 'demo', directory, fname))

