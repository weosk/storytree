
// Sorts children
fn dive_to_sort(index: usize, branches: &mut Vec<Branch>) { 

    if branches.len() > index && branches[index].children.len() != 0 {
        let mut sort_vec: Vec<(usize,i32)> = vec![];

        for i in 0..branches[index].children.len() {
            // if index == 0 {
            //     info!("i: {:?}, branches[index].children.len(): {:?}",i,branches[index].children.len());
            // }

            if branches[index].children[i] < branches.len() {

                sort_vec.push((branches[index].children[i], branches[branches[index].children[i]].num_of_all_children)); // Child indizes der Reihe nach
            }
            // if index == 0 {
            //     info!("sortvec: {:?} \n {:?}",sort_vec,sort_vec[0].1 );
            // }
        }
    
        // if index == 0 {
        //     info!("1 Index: {:?}, {:?} len() ", index, branches[index].children.len());
        //     for j in branches[index].children.clone() {
        //         info!("j: {:?} # {:?}",j, branches[j].num_of_all_children);
        //     }
        // }

        if sort_vec.len() > 0 {
            // info!("1SortVec: {:?}",sort_vec);
            sort_vec.sort_by_key(|k| k.1);
            // Standart is highest value on last position
            // sort_vec.reverse();

            // info!("2SortVec: {:?}",sort_vec);
            
            // if index == 0 {
            //     info!("\nbranches[index].children.len(): {:?}, \nsort_vec.len(): {:?}",branches[index].children.len(), sort_vec.len());
            // }
            for i in 0..sort_vec.len() {
                // if index == 0 {
                //     info!("[[]] sort_vec[i].0: {:?}, 1: {:?}, #ofchildren: {:?}",sort_vec[i].0, sort_vec[i].1, branches[branches[index].children[i]].num_of_all_children);
                // }
                    branches[index].children[i] = sort_vec[i].0;
                // if index == 0 {
                //     info!("[[]] sort_vec[i].0: {:?}, 1: {:?}, #ofchildren: {:?}",sort_vec[i].0, sort_vec[i].1, branches[branches[index].children[i]].num_of_all_children);
                // }
            }
            // branches[index].children.reverse();
            // if index == 0 {
            //     info!("2branches.children: {:?}",branches[index].children);
            // }
        }

        // if index == 0 {
        //     info!("2 Index: {:?}, {:?} len() ", index, branches[index].children.len());
        //     for j in branches[index].children.clone() {
        //         info!("j: {:?} # {:?}",j, branches[j].num_of_all_children);
        //     }
        // }

        for child_index in branches[index].children.clone() {
            if branches.len() > child_index {
                dive_to_sort(child_index, branches);
            }
        }
    }
}

