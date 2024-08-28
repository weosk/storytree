
    // Names to parameters to generate different shaped trees from names

    // Can we get this in parametrices?
    // Names can change over time?
    fn dive(name:&mut str, index: usize, branches: &mut Vec<Branch>, line_vertices: &mut Vec<Vec3>) -> () {
        if branches.len() > index {
            let mut pos: Vec3 = Vec3::splat(0.);
            let mut last_pos: Vec3 = Vec3::splat(0.);

            // Not used right now
            // let mut translation = Vec3{x:0.,y:1.,z:1.};
            // let mut offset = vec3(0., 0., 0.);

            // Number of vertices placed before next node is set
            let extending_factor = 20;
            // let extending_factor = name.as_bytes()[0] as i32;
            // println!("Bytes: {:?}", name.as_bytes()[0] as i32);

            let children = branches[index].children.clone();
            let mut inner_child_index: usize = 0;

            let mut transform = branches[index].transform;
            pos = transform.translation;//transform.transform_point(Vec3::splat(0.)); 
            last_pos = pos;

            let mut spiral_pos = Vec3::splat(0.);
            let mut spiral_transform = Transform::default();

            // spiral_transform.translation.y = 0.2;
            // spiral_transform.translation.z = 0.06;
            // spiral_transform.translati!on.y = 1.;
            // spiral_transform.translation.z = 0.333;
            spiral_transform.translation.y = 0.4;
            spiral_transform.translation.z = 0.12;
            spiral_transform.rotate_y(PI/16.);

            // range pi
            // spiral_transform.rotate_y(name.as_bytes()[1] as f32);

            // println!(" Path:{:?}\n Index: {:?}\n ChildrenLen:{:?}\n Children:{:?}\n\n",branches[index].name, index, branches[index].children.len(), branches[index].children );

            for i in 0..(branches[index].children.len() as i32 * extending_factor) - 0 { // Number of vertices of branch
                // if spiral_transform.translation.z <= 0.5 {
                //     spiral_transform.translation.z += 0.1;
                // }
                spiral_pos = spiral_transform.transform_point(spiral_pos);
                
                // Not needed apparently
                // pos = Transform::from_scale(transform.scale).transform_point(spiral_pos);
                
                pos = Transform::from_rotation(transform.rotation).transform_point(spiral_pos);
                pos = Transform::from_translation(transform.translation).transform_point(pos);
                
                // Place Transform // Put node
                if i % extending_factor == extending_factor - 1 {
                    if branches.len() > children[inner_child_index] { // To prevent len == index for /
                        let dir = pos - last_pos;
                        
                        let mut rts = spiral_transform;

                        // Original: 
                        // rts.look_to(dir.any_orthonormal_vector(), dir);
                        // rts.look_to(Vec3 { x: 0., y: 1., z: 0. } + pos, dir);
                        if branches[index].depth < 1 {
                            rts.look_to(dir, Vec3{ x: 0., y: 1., z: 0. });
                            rts = rts.with_translation(pos);
                            rts = rts.with_scale(Vec3::splat(2.));
                        }
                        else if branches[index].depth < 3 {
                            rts.look_to(dir, Vec3{ x: 1., y: 0., z: 0. });
                            rts = rts.with_translation(pos);
                            rts = rts.with_scale(Vec3::splat(1.));
                        }
                        else if branches[index].depth < 5 {
                            rts.look_to(dir, Vec3{ x: 0., y: 0., z: 1. });
                            rts = rts.with_translation(pos);
                            rts = rts.with_scale(Vec3::splat(0.7));
                        }
                        else if branches[index].depth < 7 {
                            rts.look_to(dir, Vec3{ x: 0., y: 1., z: 0. });
                            rts = rts.with_translation(pos);
                            rts = rts.with_scale(Vec3::splat(0.3));
                        }
                        // rts.rotate_y(PI);

                        // Original:
                        // rts.look_to(dir, Vec3{ x: 0., y: 1., z: 0. });
                        // rts = rts.with_translation(pos);
                        // rts = rts.with_scale(Vec3::splat(3.-(0.5*branches[index].depth as f32)));

                        // rts = rts.with_scale(Vec3::splat(10.));
                        branches[children[inner_child_index]].transform = rts;
                        inner_child_index += 1;
                    }
                }
                    line_vertices.push(last_pos);
                    line_vertices.push(pos);
                    last_pos = pos;
            }

            for child_index in branches[index].children.clone() {
                if !(branches.len() < child_index) {
                    // println!("ChildIndex: {:?} \nBranchesLen: {:?}", child_index, branches.len());
                    dive(name, child_index, branches, line_vertices);
                }
            }
        }
    }



//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////// UPDATE 15.07.2024 WITH MOST COMMANDS //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////



fn dive(name:&mut str, index: usize, branches: &mut Vec<Branch>, line_vertices: &mut Vec<Vec3>) -> () {

    // if index <= 100 {
    //     info!("\nIndex : {:?}\nParent: {:?}\nSiblings: {:?}",index, branches[index].parent, branches[branches[index].parent].children);   
    // }
    if branches.len() > index {
        let mut pos: Vec3 = Vec3::splat(0.);
        let mut last_pos: Vec3 = Vec3::splat(0.);
        let mut extending_factor = 40;//20;//branches[index].children;//20;
        let children = branches[index].children.clone();
        let mut inner_child_index: usize = 0;

        let mut transform = branches[index].transform;
        pos = transform.translation;//transform.transform_point(Vec3::splat(0.)); 
        last_pos = pos;

        let mut spiral_pos = Vec3::splat(0.);
        let mut spiral_transform = Transform::default();

        let mut scale = 2.;//0.1*(10. - branches[index].depth as f32 * 3.);

        // if branches[index].num_of_all_children > 0 && {
        //     info!("Number of All Subfolders of #{:?} : {:?}",index, branches[index].num_of_all_children );
        // }    


        // if branches[index].depth <= 1 {
        //     scale = 3.;
        //     extending_factor = 100;
        //     spiral_transform.translation.y = 10.1 ;//* scale;
        //     spiral_transform.translation.z = 30.333;// * scale;
        //     spiral_transform.rotate_y(PI/16.);
        // }
        // else if branches[index].depth <= 2 {
        //     scale = 0.5;
        //     extending_factor = 20;
        //     spiral_transform.translation.y = 3.1 ;//* scale;
        //     spiral_transform.translation.z = 1.333;// * scale;
        //     spiral_transform.rotate_y(PI/16.);
        // }
        // else if branches[index].depth <= 3 {
        //     extending_factor = 40;
        //     spiral_transform.translation.y = 1.0 ;//* scale;
        //     spiral_transform.translation.z = 3.333;// * scale;
        //     spiral_transform.rotate_y(PI/16.);
        // }
        // else if branches[index].depth <= 5 {
        //     extending_factor = 35;
        //     spiral_transform.translation.y = 1.0 ;//* scale;
        //     spiral_transform.translation.z = 3.333;// * scale;
        //     spiral_transform.rotate_y(PI/16.);
        // }
        // if branches[index].depth <= 1 {
        //     scale = 10.;
        //     extending_factor = 20 * scale as i32;
        //     spiral_transform.translation.y = 0.5 * scale * 10.;//* scale;
        //     spiral_transform.translation.z = 0.333 * scale * 1000.;// * scale;
        //     spiral_transform.rotate_y(PI/16.);
        //     // spiral_transform.rotate_y(1.*PI/(index as f32+0.1) as f32);
        // }
        // else if branches[index].depth <= 2 {
        //     scale = 5.;
        //     extending_factor = 20 * scale as i32;
        //     spiral_transform.translation.y = 0.5 * scale ;//* scale;
        //     spiral_transform.translation.z = 0.333 * scale * 100.;// * scale;
        //     spiral_transform.rotate_y(PI/16.);
        //     // spiral_transform.rotate_y(1.*PI/(index as f32+0.1) as f32);
        // }
        // else if branches[index].depth <= 3 {
        //     scale = 2.;
        //     extending_factor = 20 * scale as i32;
        //     spiral_transform.translation.y = 0.5 * scale;//* scale;
        //     spiral_transform.translation.z = 0.333 * scale * 10.;// * scale;
        //     spiral_transform.rotate_y(PI/16.);
        //     // spiral_transform.rotate_y(1.*PI/(index as f32+0.1) as f32);
        // }

        if false {
            // spiral_transform.translation.y = 0.5 * scale;//* scale;
            // spiral_transform.translation.z = 0.333 * scale * 10.;// * scale;
            // spiral_transform.rotate_y(PI/16.);
        }
        else { // "Natural"
            // scale = 10. - branches[index].depth as f32 * 2.;
            // scale = 3.;//1.;//3.;//10. - branches[index].depth as f32 * 2.;
            scale = 10. -0.5 * branches[index].depth as f32;
            // scale = 1.;
            extending_factor = branches[index].num_of_all_children+1;//70;//10 *scale as i32;
            if extending_factor > 30 {
                extending_factor = 30;
            }

            // spiral_transform.translation.z = (0.333 + index as f32 *scale);// * i;// * scale;
            // spiral_transform.translation.y = (1.1   +  branches[index].depth as f32) *scale;// * 0.3*i;//* scale;

            // spiral_transform.translation.z = 1.333 *scale;// * i;// * scale;
            // spiral_transform.translation.y = 1.1   *scale;// * 0.3*i;//* scale;


            // spiral_transform.translation.z = -50.;// * i;// * scale;
            // spiral_transform.translation.x = -50.;//1.1   *scale;// * 0.3*i;//* scale;


            // Standart swirl
            // spiral_transform.rotate_y(PI/128.);
            // spiral_transform.rotate_y(PI/16.);

            // Plantlike
            // spiral_transform.rotate_y(4.*PI/(index as f32+0.5) as f32);

            // Spiral swirl
            // spiral_transform.rotate_y();

        }



        // else if branches[index].depth <= 2 {
        //     extending_factor = 50;
        //     spiral_transform.translation.y = 2.2 * scale;
        //     spiral_transform.translation.z = 15.333 * scale;
        //     // spiral_transform.rotate_y(4.*PI/(index as f32+0.5) as f32);
        //     spiral_transform.rotate_y(PI/16.);
        // }
        // else if branches[index].depth <= 5 {
        //     extending_factor = 40;
        //     spiral_transform.translation.y = 1.3 * scale;
        //     spiral_transform.translation.z = 7.333 * scale;
        //     // spiral_transform.rotate_y(4.*PI/(index as f32+0.5) as f32);
        //     spiral_transform.rotate_y(PI/16.);
        // }
        // else if branches[index].depth <= 6{
        //     extending_factor = 30;
        //     spiral_transform.translation.y = 0.5 * scale;
        //     spiral_transform.translation.z = 3.333 * scale;
        //     // spiral_transform.rotate_y(4.*PI/(index as f32+0.5) as f32);
        //     spiral_transform.rotate_y(PI/16.);
        // }
        // else if branches[index].depth <= 10{
        //     extending_factor = 20;

        //     spiral_transform.translation.y = 0.5 * scale;
        //     spiral_transform.translation.z = 3.333 * scale;
        //     spiral_transform.rotate_y(4.*PI/(index as f32+0.5) as f32);
        //     // spiral_transform.rotate_y(PI/16.);
        // }
            


        // ProvenTrees
        // extending_factor = 40;
        // spiral_transform.translation.y = 1. * scale;
        // spiral_transform.translation.z = 0.333 * scale;

        // Fixed variant for small data:
        // spiral_transform.rotate_y(PI/16.);

        // Variable variant for big data:
        // spiral_transform.rotate_y(4.*PI/(index as f32+0.5) as f32);

        let scale = Vec3::splat(scale);
        let mut z = 0;

        let vertex_iteration = (branches[index].children.len() as i32 * extending_factor) - 0;
        for i in 0..vertex_iteration { // Number of vertices of branch

            // spiral_transform.translation.x =scale.z* i as f32 *3.;
            // spiral_transform.translation.y =i as f32;//scale.y* 1.*(1.-E.powf(-1.*i as f32));//+= 0.01;//1.0   *scale.y * 0.3 * (E).powi(i);//* scale;
            // if i < branches[index].children.len() as i32 * extending_factor /2 {
            //     z = -i;
            // }
            // else {
            //     z = i - branches[index].children.len() as i32 * extending_factor /2;
            // }
            // spiral_transform.translation.z =scale.z* i as f32 *3.;//1.2;//0.333 *scale.z * i as f32;// * scale;
            
            // if branches[index].depth == 0{// % 2 == 0 { 
            if index == 0 {

                spiral_transform.translation.x =0.7*scale.x *  i as f32 *(i as f32 * PI/27.).cos(); //8.*
                spiral_transform.translation.y =1.*scale.y *(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
                spiral_transform.translation.z =0.7*scale.z *  i as f32  *(i as f32 * PI/27.).sin();//8.*
                // extending_factor = 100;
            } 
            else if index == *branches[branches[index].parent].children.last().unwrap() {
                // info!("DepthLast: {:?} Index: {:?}", branches[index].depth, index);

                let scale = Vec3::splat(0.1);
                spiral_transform.translation.x =1.*scale.x *  i as f32 *(i as f32 * PI/16.).cos(); //8.*
                spiral_transform.translation.y =branches[index].depth as f32*1.+1.*(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
                spiral_transform.translation.z =-0.1*i as f32+1.*scale.z   *(i as f32 * PI/16.).sin();//8.*
                // extending_factor = 100;
            }  
            
            
            //2
            // else if branches[index].depth % 2 == 0 {
            //     spiral_transform.translation.x =0.1*scale.x *  i as f32 *(i as f32 * PI/16.).cos(); //8.*
            //     spiral_transform.translation.y =1. *(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
            //     spiral_transform.translation.z =0.1*scale.z *  i as f32  *(i as f32 * PI/16.).sin() +100.;//8.*
            // }
            // funnel
            // else if branches[index].depth % 2 == 0 {
            //     spiral_transform.translation.x =0.01*scale.x *  i as f32 *(i as f32 * PI/16.).cos(); //8.*
            //     spiral_transform.translation.y =1. *(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
            //     spiral_transform.translation.z =0.01*scale.z *  i as f32  *(i as f32 * PI/16.).sin();//8.*   
            // }
            // else if branches[index].depth % 2 == 0 {
            //     spiral_transform.translation.x =0.01*scale.x *  1 as f32 *(i as f32 * PI/16.).cos(); //8.*
            //     spiral_transform.translation.y =0.1 *(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
            //     spiral_transform.translation.z =0.01* scale.z *  i as f32  *(i as f32 * PI/16.).sin();//8.*   
            // }
            else {//1
                // spiral_transform.translation.x =scale.x ;//*  i as f32 *(i as f32 * PI/16.).cos(); //8.*
                // spiral_transform.translation.y =10.+scale.y ;//* 2.*(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
                // spiral_transform.translation.z =scale.z ;//*  i as f32  *(i as f32 * PI/16.).sin();//8.*

                spiral_transform.translation.x =0.1*i as f32 *scale.x *(i as f32 * PI/16.).cos(); //8.*
                spiral_transform.translation.y =0.2*scale.y *(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
                spiral_transform.translation.z =-0.01*i as f32 + 0.5*scale.z *(i as f32 * PI/16.).sin();//8.*
            }

            // Spiraling Up
            spiral_pos = spiral_transform.transform_point(spiral_pos);
            // Rotate into formerly given direction
            pos = Transform::from_rotation(transform.rotation).transform_point(spiral_pos);
            // Translate to formerly given position
            pos = Transform::from_translation(transform.translation).transform_point(pos);

            // Assigning the node
            if i % extending_factor == extending_factor - 1 {
                if branches.len() > children[inner_child_index] { // To prevent len == index for /
                    let dir = pos - last_pos;
                    
                    let mut rts = spiral_transform;

                    // Branch of from pos with last pos to pos direction
                    // rts.look_to(dir.normalize().any_orthonormal_vector(), dir);

                    // info!("\nIndex : {:?}\nParent: {:?}\nSiblings: {:?}",index, branches[index].parent, branches[branches[index].parent].children);
                    if index == *branches[branches[index].parent].children.last().unwrap() {
                        // rts = rts.with_translation(branches[branches[index].parent].transform.translation + Vec3 {x: 0., y: 1000., z: 0. });
                        rts.look_to( pos, Vec3 {x: 0., y: 1., z: 0. });
                        rts = rts.with_translation(pos);
                    }
                    else {
                        rts.look_to(pos, Vec3 {x: 0., y: 1., z: 0. });
                        rts = rts.with_translation(pos);
                    }      
                    rts = rts.with_scale(scale);
                    // }
                    branches[children[inner_child_index]].transform = rts;
                    inner_child_index += 1;
                }
            }
                line_vertices.push(last_pos);
                line_vertices.push(pos);
                last_pos = pos;
        }

        for child_index in branches[index].children.clone() {
            if !(branches.len() < child_index) {
                // println!("ChildIndex: {:?} \nBranchesLen: {:?}", child_index, branches.len());
                dive(name, child_index, branches, line_vertices);
            }
        }
    }
}
